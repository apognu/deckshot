#!/bin/sh

set -e

if [ "$(id -nu)" != 'deck' ]; then
  echo "ERROR: this should be run as the 'deck' user on a Steam Deck." >&2
  exit 1
fi

DECKSHOT_DIR='/home/deck/.local/share/deckshot'

systemctl --user disable --now deckshot

rm -rf $DECKSHOT_DIR
rm /home/deck/.config/systemd/user/deckshot.service

systemctl --user daemon-reload
