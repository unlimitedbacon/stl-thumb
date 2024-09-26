%define __spec_install_post %{nil}
%define __os_install_post %{_dbpath}/brp-compress
%define debug_package %{nil}

Name: stl-thumb
Summary: A fast lightweight thumbnail generator for 3D model(STL, OBJ, 3MF) files
Version: @@VERSION@@
Release: @@RELEASE@@%{?dist}
License: MIT
Group: Applications/System
Source0: %{name}-%{version}.tar.gz
URL: https://github.com/unlimitedbacon/stl-thumb

BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root

%description
%{summary}

%prep
%setup -q

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
cp -a * %{buildroot}

%clean
rm -rf %{buildroot}

%files
%defattr(-,root,root,-)
%{_bindir}/stl-thumb
%{_includedir}/libstl_thumb.h
%{_libdir}/libstl_thumb.a
%{_libdir}/libstl_thumb.so
%{_datadir}/thumbnailers/stl-thumb.thumbnailer
%{_datadir}/mime/packages/stl-thumb-mime.xml
%{_datadir}/doc/stl-thumb/README.md

