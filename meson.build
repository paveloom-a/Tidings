project(
  'tidings',
  'rust',
  license: 'GPL-3.0-only',
  meson_version: '>= 0.59.4',
  version: run_command(
    'grep',
    '-oPm',
    '1',
    'version = "\K[^"]+',
    'Cargo.toml',
    check: true
  ).stdout().strip(),
)

year = '2022'
date = '2022-06-25'

i18n = import('i18n')
gnome = import('gnome')

app_name = 'tidings'
app_name_capitalized = 'Tidings'
base_id = 'paveloom.apps.' + app_name

dependency('glib-2.0', version: '= 2.72.3')
dependency('gio-2.0', version: '= 2.72.3')
dependency('gtk4', version: '= 4.6.6')
dependency('libadwaita-1', version: '= 1.1.3')

cargo = find_program('cargo', required: true)
cargo_script = find_program(meson.project_source_root() / 'scripts' / 'build.bash')
sources_script =find_program(meson.project_source_root() / 'scripts' / 'sources.bash')
glib_compile_resources = find_program('glib-compile-resources', required: true)
glib_compile_schemas = find_program('glib-compile-schemas', required: true)
desktop_file_validate = find_program('desktop-file-validate', required: false)
appstream_util = find_program('appstream-util', required: false)
update_mime_database = find_program('update-mime-database', required: false)

version = meson.project_version()
version_array = version.split('.')
major_version = version_array[0].to_int()
minor_version = version_array[1].to_int()
version_micro = version_array[2].to_int()

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
localedir = prefix / get_option('localedir')

datadir = prefix / get_option('datadir')
pkgdatadir = datadir / meson.project_name()
iconsdir = datadir / 'icons'
domainname = meson.project_name()

if get_option('buildtype') == 'release'
  profile = 'release'
  version_suffix = ''
  application_id = '@0@'.format(base_id)
else
  profile = 'dev'
  vcs_tag = run_command('git', 'rev-parse', '--short', 'HEAD').stdout().strip()
  if vcs_tag == ''
    version_suffix = '-dev'
  else
    version_suffix = '-@0@'.format(vcs_tag)
  endif
  application_id = '@0@.@1@'.format(base_id, profile)
endif

# Install the application data
subdir('data')
# Set up gettext localisation
subdir('po')
# Prepare the source files of the application
subdir('src')

# Build the application
custom_target(
  'cargo-build',
  build_by_default: true,
  input: [rust_sources, cargo_sources],
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: bindir,
  depends: resources,
  command: [
    cargo_script,
    meson.project_build_root(),
    meson.project_source_root(),
    '@OUTPUT@',
    profile,
    meson.project_name(),
  ]
)

meson.add_install_script('scripts/post-install.bash')
