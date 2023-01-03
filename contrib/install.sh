#!/bin/sh

set -e

if [ "$(id -nu)" != 'deck' ]; then
  echo "ERROR: this should be run as the 'deck' user on a Steam Deck." >&2
  exit 1
fi

DECKSHOT_DIR='/home/deck/.local/share/deckshot'
CONFIG_FILE="${DECKSHOT_DIR}/deckshot.yml"

systemctl --user stop deckshot || true

mkdir -p /home/deck/.local/share/deckshot

curl -sL https://github.com/apognu/deckshot/releases/download/tip/deckshot-tip-x86_64 > ${DECKSHOT_DIR}/deckshot
curl -sL https://raw.githubusercontent.com/apognu/deckshot/master/contrib/deckshot.service > /home/deck/.config/systemd/user/deckshot.service

if [ ! -f $CONFIG_FILE ]; then
  curl -sL https://raw.githubusercontent.com/apognu/deckshot/master/contrib/deckshot.yml > ${CONFIG_FILE}
  chmod 0600 $CONFIG_FILE
fi

chmod u+x ${DECKSHOT_DIR}/deckshot

systemctl --user daemon-reload
systemctl --user enable --now deckshot
