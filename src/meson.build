global_conf = configuration_data()
global_conf.set_quoted('app_id', application_id)
global_conf.set_quoted('domainname', domainname)
global_conf.set_quoted('localedir', localedir)
global_conf.set_quoted('pkgdatadir', pkgdatadir)
global_conf.set_quoted('profile', profile)
global_conf.set_quoted('resources_file', pkgdatadir / 'resources.gresource')
global_conf.set_quoted('version', version + version_suffix)

config = configure_file(
  input: 'config.in',
  output: 'config.rs',
  configuration: global_conf
)

# Copy the config.rs output to the source directory.
run_command(
  'cp',
  meson.project_build_root() / 'src' / 'config.rs',
  meson.project_source_root() / 'src' / 'config.rs',
  check: true
)

rust_sources = run_command(sources_script, check: true).stdout().strip().split('\n')

cargo_sources = files(
  meson.project_source_root() / 'Cargo.toml',
  meson.project_source_root() / 'Cargo.lock',
)
