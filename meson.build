project(
  'tidings',
  'rust',
  license: 'GPL-3.0-only',
  meson_version: '>= 0.59.1',
  version: run_command(
    'grep',
    '-oPm',
    '1',
    'version = "\K[^"]+',
    'Cargo.toml',
    check: true
  ).stdout().strip(),
)

year = '2021'
date = '2021-12-30'

i18n = import('i18n')
gnome = import('gnome')

base_id = 'paveloom.apps.tidings'

dependency('glib-2.0', version: '>= 2.70.2')
dependency('gio-2.0', version: '>= 2.70.2')
dependency('gtk4', version: '>= 4.6.0')
dependency('libadwaita-1', version: '>= 1.0.0')

glib_compile_resources = find_program('glib-compile-resources', required: true)
glib_compile_schemas = find_program('glib-compile-schemas', required: true)
desktop_file_validate = find_program('desktop-file-validate', required: false)
appstream_util = find_program('appstream-util', required: false)
cargo = find_program('cargo', required: true)

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
# Build the application
subdir('src')

meson.add_install_script('scripts/post-install.bash')
