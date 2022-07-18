### Notices

#### Git mirrors

- [Codeberg](https://codeberg.org/paveloom-a/Tidings)
- [GitHub](https://github.com/paveloom-a/Tidings)
- [GitLab](https://gitlab.com/paveloom-g/apps/tidings)

#### Build

To build the application, you can use one of the [manifests](./manifests):

```bash
flatpak-builder --user --install --force-clean target/flatpak/release manifests/release.yml
```

*or*

```bash
flatpak-builder --user --install --force-clean target/flatpak/debug manifests/dev.yml
```

Additionally, these commands will install the application in your system, so you can run it.

> ***Note***: `flatpak-builder` will prune Cargo cache every time.

#### Develop

To iterate on the application, run the [`run.bash`](./run.bash) script. It replicates a part of the `flatpak-builder`'s build process with standard `flatpak` commands, allowing you to preserve the Cargo cache between builds.

> ***Note***: Make sure you have [`yq`](https://github.com/mikefarah/yq) installed.
