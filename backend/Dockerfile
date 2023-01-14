FROM ghcr.io/steamdeckhomebrew/holo-base:latest
LABEL maintainer "Antoine POPINEAU"

RUN \
  pacman -Sy --noconfirm rustup && \
  rustup install nightly-2023-01-08

ENTRYPOINT [ "/backend/entrypoint.sh" ]
