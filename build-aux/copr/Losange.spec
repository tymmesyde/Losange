Name:           Losange
Version:        0.6.0
Release:        1%{?dist}
Summary:        A simple Stremio client for GNOME

License:        GPL-3.0-only
URL:            https://github.com/tymmesyde/Losange
Source0:        https://github.com/tymmesyde/Losange/archive/v%{version}.tar.gz

BuildRequires:  cargo, rust, openssl-devel, gtk4-devel, libadwaita-devel, mpv-devel, libepoxy-devel
Requires:       nodejs >= 10

%description
Unofficial Stremio client

%prep
%setup -q

%build
export CARGO_HOME=$(pwd)/.cargo
cargo build --release

%install
mkdir -p %{buildroot}/usr/bin
install -m 755 target/release/losange %{buildroot}/usr/bin/
mkdir -p %{buildroot}/usr/share/icons/hicolor/scalable/apps
install -Dm644 data/icons/xyz.timtimtim.Losange.svg %{buildroot}/usr/share/icons/hicolor/scalable/apps/
mkdir -p %{buildroot}/usr/share/applications
install -Dm644 data/xyz.timtimtim.Losange.desktop %{buildroot}/usr/share/applications/
mkdir -p %{buildroot}/usr/share/metainfo
install -Dm644 data/xyz.timtimtim.Losange.metainfo.xml %{buildroot}/usr/share/metainfo/
mkdir -p %{buildroot}/usr/share/glib-2.0/schemas
install -Dm644 data/xyz.timtimtim.Losange.gschema.xml %{buildroot}/usr/share/glib-2.0/schemas/

%files
/usr/bin/losange
/usr/share/icons/hicolor/scalable/apps/xyz.timtimtim.Losange.svg
/usr/share/applications/xyz.timtimtim.Losange.desktop
/usr/share/metainfo/xyz.timtimtim.Losange.metainfo.xml
/usr/share/glib-2.0/schemas/xyz.timtimtim.Losange.gschema.xml

%changelog
* Fri Dec 26 2025 Tim Dusser-Jolly <tymmesyde@gmail.com> - 0.6.0-1
- Improve playback (now using MPV as internal player)
- Add subtitles position setting

* Tue Sep 28 2025 Tim Dusser-Jolly <tymmesyde@gmail.com> - 0.5.0-1
- Add genre dropdown to discover page
- Improve addons page performance
- Improved playback in some cases
- Fix discover and addons page unloading each other
- Fix aligment of placeholder icons

* Wed Mar 19 2025 Tim Dusser-Jolly <tymmesyde@gmail.com> - 0.4.1-1
- Reduce font sizes of catalog titles
- Fix audio issue with flatpak

* Sun Mar 16 2025 Tim Dusser-Jolly <tymmesyde@gmail.com> - 0.4.0-1
- Open continue watching items in player 
- Add progress to continue watching items
- Improve gradient on details page

* Sun Feb 23 2025 Tim Dusser-Jolly <tymmesyde@gmail.com> - 0.3.1-1
- Fix player track selection
- Fix player audio issues with flatpak

* Sat Feb 22 2025 Tim Dusser-Jolly <tymmesyde@gmail.com> - 0.3.0-1
- Add release year, runtime and imdb note to details page
- Use persistent data by default
- Remove storage preferences page
- Remove break line chars in stream titles

* Sat Feb 8 2025 Tim Dusser-Jolly <tymmesyde@gmail.com> - 0.2.0-2
- Fix package file

* Wed Feb 5 2025 Tim Dusser-Jolly <tymmesyde@gmail.com> - 0.2.0-1
- Add player subtitles size setting
- Fix player start time

* Sun Feb 2 2025 Tim Dusser-Jolly <tymmesyde@gmail.com> - 0.1.1-5
- Minor change to the package file

* Sat Feb 1 2025 Tim Dusser-Jolly <tymmesyde@gmail.com> - 0.1.1-4
- Minor change to the package file

* Sat Feb 1 2025 Tim Dusser-Jolly <tymmesyde@gmail.com> - 0.1.1-3
- Minor change to the package file

* Sat Feb 1 2025 Tim Dusser-Jolly <tymmesyde@gmail.com> - 0.1.1-2
- Fix package file

* Sat Feb 1 2025 Tim Dusser-Jolly <tymmesyde@gmail.com> - 0.1.1-1
- Initial package