#! /bin/bash

exec journalctl -f --user-unit=elkr-engine.service --user-unit=elkr-manager.service
