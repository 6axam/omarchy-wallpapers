#!/usr/bin/env python3
"""Rotate user-supplied wallpapers in Omarchy at random intervals."""
import json
import logging
import os
import random
import signal
import subprocess
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CONFIG = Path(os.environ.get('OMARCHY_WALLPAPERS_CONFIG', Path.home() / '.config/omarchy-wallpapers/settings.json')).expanduser()
STATE = Path(os.environ.get('XDG_RUNTIME_DIR', '/run/user/' + str(os.getuid()))) / 'random-wallpaper-state.json'
EXTENSIONS = {'.jpg', '.jpeg', '.png', '.webp', '.bmp'}
logging.basicConfig(level=logging.INFO, format='%(asctime)s %(message)s')
shuffle_requested = False

def request_shuffle(signum, frame):
    global shuffle_requested
    shuffle_requested = True


def settings():
    data = json.loads(CONFIG.read_text(encoding='utf-8'))
    low, high = float(data['min_minutes']), float(data['max_minutes'])
    if not 0 < low <= high:
        raise ValueError('Intervals must be positive and min_minutes <= max_minutes')
    return Path(data['folder']).expanduser(), low * 60, high * 60


def write_state(status, deadline=None):
    payload = {'status': status, 'deadline': deadline, 'pid': os.getpid()}
    tmp = STATE.with_suffix('.tmp')
    tmp.write_text(json.dumps(payload), encoding='utf-8')
    tmp.replace(STATE)


def main():
    global shuffle_requested
    signal.signal(signal.SIGUSR1, request_shuffle)
    previous = None
    remaining = []
    deadline = 0.0
    first_run = True
    last_state = None
    while True:
        try:
            folder, low, high = settings()
            folder.mkdir(parents=True, exist_ok=True)
            images = sorted(p.resolve() for p in folder.iterdir()
                            if p.is_file() and p.suffix.lower() in EXTENSIONS)
            if first_run:
                current = Path.home() / '.local/state/omarchy/current/background'
                if current.is_symlink() and current.resolve() in images:
                    previous = current.resolve()
                    deadline = time.monotonic() + random.uniform(low, high)
                first_run = False
            remaining = [p for p in remaining if p in images]
            if shuffle_requested:
                remaining = images.copy()
                random.shuffle(remaining)
                if len(remaining) > 1 and remaining[-1] == previous:
                    remaining[0], remaining[-1] = remaining[-1], remaining[0]
                if images:
                    deadline = time.monotonic() + random.uniform(low, high)
                shuffle_requested = False
                logging.info('Очередь перемешана, время следующей смены обновлено')
            if images and time.monotonic() >= deadline:
                if not remaining:
                    remaining = images.copy()
                    random.shuffle(remaining)
                    if len(remaining) > 1 and remaining[-1] == previous:
                        remaining[0], remaining[-1] = remaining[-1], remaining[0]
                chosen = remaining.pop()
                subprocess.run(['omarchy', 'theme', 'bg', 'set', str(chosen)], check=True)
                previous = chosen
                delay = random.uniform(low, high)
                deadline = time.monotonic() + delay
                logging.info('Обои: %s; следующая смена через %.1f мин.', chosen.name, delay / 60)
            elif not images:
                deadline = 0.0
            state = ('scheduled', deadline) if images else ('empty', None)
        except (OSError, ValueError, KeyError, json.JSONDecodeError, subprocess.SubprocessError) as exc:
            logging.error('Не удалось сменить обои: %s', exc)
            deadline = time.monotonic() + 60
            state = ('error', None)
        try:
            if state != last_state:
                write_state(*state)
                last_state = state
        except OSError as exc:
            logging.error('Не удалось сохранить состояние таймера: %s', exc)
        time.sleep(1)


if __name__ == '__main__':
    main()
