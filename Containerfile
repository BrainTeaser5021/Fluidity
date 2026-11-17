FROM ghcr.io/ublue-os/kinoite-main:latest

LABEL org.opencontainers.image.title="Fluidity OS" \
      org.opencontainers.image.description="Fedora Kinoite base with KWin liquid-glass compositor, Rust desktop shell, and local 0.5B-1.5B AI launcher"

# 1. Install build & runtime dependencies for Rust Wayland shell, Flatpak, and Distrobox
RUN dnf5 install -y \
    distrobox \
    podman \
    flatpak \
    kwin-wayland \
    qt6-qtwayland \
    layer-shell-qt \
    wayland-devel \
    libxkbcommon-devel \
    mesa-dri-drivers \
    && dnf5 clean all

# 2. Copy pre-built Fluidity Rust binary built by GitHub Actions
COPY target/release/fluidity-launcher /usr/bin/fluidity-launcher

# 3. Install KWin Puddle-Close Shader Effect system-wide
COPY kwin-puddle-effect /usr/share/kwin/effects/kwin4_effect_fluidity_puddle/

# 4. Configure Fluidity Wayland session (uses KWin compositor + Fluidity Rust shell instead of plasmashell)
RUN mkdir -p /usr/share/wayland-sessions /usr/lib/systemd/user && \
    printf '[Desktop Entry]\nName=Fluidity\nComment=Fluidity Liquid Glass Rust Desktop\nExec=/usr/bin/startplasma-wayland\nType=Application\nDesktopNames=KDE\n' > /usr/share/wayland-sessions/fluidity.desktop

# 5. Mask plasmashell in user systemd so Fluidity's Rust panel/dock/launcher take over
RUN ln -sf /dev/null /usr/lib/systemd/user/plasma-plasmashell.service

# 6. Enable Flathub by default for sandboxed container apps
RUN mkdir -p /etc/flatpak/remotes.d && \
    curl -fsSL https://dl.flathub.org/repo/flathub.flatpakrepo -o /etc/flatpak/remotes.d/flathub.flatpakrepo || true

RUN ostree container commit
