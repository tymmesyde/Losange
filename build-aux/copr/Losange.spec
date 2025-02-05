Name:           Losange
Version:        0.1.1
Release:        5%{?dist}
Summary:        A simple Stremio client for GNOME

License:        GPL-3.0-only
URL:            https://github.com/tymmesyde/Losange
Source0:        https://github.com/tymmesyde/Losange/archive/v%{version}.tar.gz

BuildRequires:  cargo, rust, openssl-devel, gtk4-devel, libadwaita-devel, gstreamer1-devel, gstreamer1-plugins-base-devel
Requires:       node >= 10

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