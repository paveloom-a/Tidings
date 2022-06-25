subdir('icons')
subdir('resources')

# Get the path to the directory with translations
po_dir = meson.project_source_root() / 'po'

# Desktop file
desktop_conf = configuration_data()
desktop_conf.set('icon', application_id)
desktop_file = i18n.merge_file(
  type: 'desktop',
  input: configure_file(
    input: '@0@.desktop.in.in'.format(base_id),
    output: '@BASENAME@',
    configuration: desktop_conf
  ),
  output: '@0@.desktop'.format(application_id),
  po_dir: po_dir,
  install: true,
  install_dir: datadir / 'applications'
)
# Validate Desktop file
if desktop_file_validate.found()
  test(
    'validate-desktop',
    desktop_file_validate,
    args: [
      desktop_file.full_path()
    ],
    depends: desktop_file,
  )
endif

# Appdata
appdata_conf = configuration_data()
appdata_conf.set('app-name-capitalized', app_name_capitalized)
appdata_conf.set('app-id', application_id)
appdata_conf.set('date', date)
appdata_conf.set('domainname', domainname)
appdata_conf.set('version', version + version_suffix)
appdata_conf.set('year', year)
appdata_file = i18n.merge_file(
  input: configure_file(
    input: '@0@.metainfo.xml.in.in'.format(base_id),
    output: '@BASENAME@',
    configuration: appdata_conf
  ),
  output: '@0@.metainfo.xml'.format(application_id),
  po_dir: po_dir,
  install: true,
  install_dir: datadir / 'metainfo'
)
# Validate Appdata
if appstream_util.found()
  test(
    'validate-appdata', appstream_util,
    args: [
      'validate', '--nonet', appdata_file.full_path()
    ],
    depends: appdata_file,
  )
endif

# GSchema
gschema_conf = configuration_data()
gschema_conf.set('app-id', application_id)
gschema_conf.set('domainname', domainname)
configure_file(
  input: '@0@.gschema.xml.in'.format(base_id),
  output: '@0@.gschema.xml'.format(application_id),
  configuration: gschema_conf,
  install: true,
  install_dir: datadir / 'glib-2.0' / 'schemas'
)
# Validata GSchema
if glib_compile_schemas.found()
  test(
    'validate-gschema', glib_compile_schemas,
    args: [
      '--strict', '--dry-run', meson.current_build_dir()
    ],
  )
endif
