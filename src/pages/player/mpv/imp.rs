use libc::{setlocale, LC_NUMERIC};
use libmpv2::{
    events::{Event, PropertyData},
    render::{OpenGLInitParams, RenderContext, RenderParam, RenderParamApiType},
    Format, Mpv, SetData,
};
use relm4::gtk::{
    self, TickCallbackId, gdk::GLContext, glib::{self, ControlFlow, Propagation, Properties, SourceId, Variant, clone, subclass::*}, prelude::*, subclass::prelude::*
};
use std::{
    cell::{Cell, RefCell},
    env,
    os::raw::c_void,
    sync::OnceLock,
};
use tracing::error;

fn get_proc_address(_context: &GLContext, name: &str) -> *mut c_void {
    epoxy::get_proc_addr(name) as _
}

#[derive(Properties)]
#[properties(wrapper_type = super::MpvPlayer)]
pub struct MpvPlayer {
    #[property(get, set)]
    scale_factor: Cell<i32>,
    mpv: RefCell<Mpv>,
    render_context: RefCell<Option<RenderContext>>,
    events_source: RefCell<Option<SourceId>>,
    tick_callback: RefCell<Option<TickCallbackId>>,
    fbo: Cell<u32>,
    width: Cell<i32>,
    height: Cell<i32>,
}

impl Default for MpvPlayer {
    fn default() -> Self {
        // Required for libmpv to work alongside gtk
        unsafe {
            setlocale(LC_NUMERIC, c"C".as_ptr());
        }

        let log = env::var("RUST_LOG");
        let msg_level = match log {
            Ok(scope) => &format!("all={}", scope.as_str()),
            _ => "all=no",
        };

        let mpv = Mpv::with_initializer(|init| {
            init.set_option("vo", "libmpv")?;
            init.set_option("hwdec", "auto")?;
            init.set_option("video-sync", "audio")?;
            init.set_option("video-timing-offset", "0")?;
            init.set_option("terminal", "yes")?;
            init.set_option("msg-level", msg_level)?;
            Ok(())
        })
        .expect("Failed to create mpv");

        mpv.disable_deprecated_events().ok();

        Self {
            scale_factor: Cell::new(1),
            mpv: RefCell::new(mpv),
            render_context: Default::default(),
            events_source: Default::default(),
            tick_callback: Default::default(),
            fbo: Default::default(),
            width: Default::default(),
            height: Default::default(),
        }
    }
}

impl MpvPlayer {
    fn fbo(&self) -> i32 {
        let mut fbo = self.fbo.get();

        if fbo == 0 {
            let mut current_fbo = 0;

            unsafe {
                epoxy::GetIntegerv(epoxy::FRAMEBUFFER_BINDING, &mut current_fbo);
            }

            fbo = current_fbo as u32;
            self.fbo.set(fbo);
        }

        fbo as i32
    }

    fn on_event<T: Fn(Event)>(&self, callback: T) {
        if let Some(result) = self.mpv.borrow_mut().wait_event(0.0) {
            match result {
                Ok(event) => callback(event),
                Err(e) => error!("Failed to wait for event: {e}"),
            }
        };
    }

    pub fn send_command(&self, name: &str, args: &[&str]) {
        if let Err(e) = self.mpv.borrow().command(name, args) {
            error!("Failed to send command {name}: {e}");
        }
    }

    pub fn observe_property(&self, name: &str, format: Format) {
        if let Err(e) = self.mpv.borrow().observe_property(name, format, 0) {
            error!("Failed to observe property {name}: {e}");
        }
    }

    pub fn set_property<T: SetData>(&self, name: &str, value: T) {
        if let Err(e) = self.mpv.borrow().set_property(name, value) {
            error!("Failed to set property {name}: {e}");
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for MpvPlayer {
    const NAME: &'static str = "MpvPlayer";
    type Type = super::MpvPlayer;
    type ParentType = gtk::GLArea;
}

#[glib::derived_properties]
impl ObjectImpl for MpvPlayer {
    fn signals() -> &'static [Signal] {
        static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
        SIGNALS.get_or_init(|| {
            vec![
                Signal::builder("property-changed")
                    .param_types([str::static_type(), Variant::static_type()])
                    .build(),
                Signal::builder("playback-ended").build(),
                Signal::builder("playback-error").build(),
            ]
        })
    }
}

impl WidgetImpl for MpvPlayer {
    fn map(&self) {
        self.parent_map();

        let object = self.obj();
        object.make_current();

        if object.error().is_some() {
            return;
        }

        if let Some(context) = object.context() {
            let mut mpv = self.mpv.borrow_mut();
            let handle = unsafe { mpv.ctx.as_mut() };

            let params = vec![
                RenderParam::ApiType(RenderParamApiType::OpenGl),
                RenderParam::InitParams(OpenGLInitParams {
                    get_proc_address,
                    ctx: context,
                }),
                RenderParam::BlockForTargetTime(false),
            ];

            let render_context =
                RenderContext::new(handle, params).expect("Failed to create render context");

            *self.render_context.borrow_mut() = Some(render_context);

            let tick_callback = object.add_tick_callback(clone!(
                #[weak]
                object,
                #[upgrade_or]
                ControlFlow::Continue,
                move |_, _| {
                    object.queue_render();
                    ControlFlow::Continue
                }
            ));

            *self.tick_callback.borrow_mut() = Some(tick_callback);

            let events_source = glib::idle_add_local(clone!(
                #[weak(rename_to = mpv_player)]
                self,
                #[weak]
                object,
                #[upgrade_or]
                ControlFlow::Continue,
                move || {
                    mpv_player.on_event(|event| match event {
                        Event::PropertyChange { name, change, .. } => {
                            let value = match change {
                                PropertyData::Str(v) => Some(v.to_variant()),
                                PropertyData::Flag(v) => Some(v.to_variant()),
                                PropertyData::Int64(v) => Some(v.to_variant()),
                                PropertyData::Double(v) => Some(v.to_variant()),
                                _ => None,
                            };

                            if let Some(value) = value {
                                object.emit_by_name::<()>("property-changed", &[&name, &value]);
                            }
                        }
                        Event::EndFile(reason) => {
                            if reason == 0 {
                                object.emit_by_name::<()>("playback-ended", &[]);
                            }

                            if reason == 4 {
                                object.emit_by_name::<()>("playback-error", &[]);
                            }
                        }
                        _ => {}
                    });

                    ControlFlow::Continue
                }
            ));

            self.events_source.borrow_mut().replace(events_source);
        }
    }

    fn unmap(&self) {
        if let Some(events_source) = self.events_source.borrow_mut().take() {
            events_source.remove();
        }

        if let Some(tick_callback) = self.tick_callback.borrow_mut().take() {
            tick_callback.remove();
        }

        if let Some(render_context) = self.render_context.borrow_mut().take() {
            drop(render_context);
        }

        self.parent_unmap();
    }
}

impl GLAreaImpl for MpvPlayer {
    fn render(&self, _context: &GLContext) -> Propagation {
        let fbo = self.fbo();
        let scale_factor = self.scale_factor.get();
        let width = self.width.get();
        let height = self.height.get();

        if let Some(ref render_context) = *self.render_context.borrow() {
            render_context
                .render::<GLContext>(fbo, width * scale_factor, height * scale_factor, true)
                .expect("Failed to render");
        }

        Propagation::Stop
    }

    fn resize(&self, width: i32, height: i32) {
        self.width.set(width);
        self.height.set(height);
    }
}
