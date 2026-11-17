#!/usr/bin/env python3
import json
import sys
import shutil
import math
import os
import time
import random
import subprocess
import tkinter as tk
from pathlib import Path
from tkinter import messagebox
from PIL import Image, ImageOps, ImageTk
sys.path.insert(0, str(Path(__file__).resolve().parent / 'vendor'))
from tkinterdnd2 import DND_FILES, TkinterDnD

CONFIG = Path(os.environ.get('OMARCHY_WALLPAPERS_CONFIG', Path.home() / '.config/omarchy-wallpapers/settings.json')).expanduser()
STATE = Path(os.environ.get('XDG_RUNTIME_DIR', '/run/user/' + str(os.getuid()))) / 'random-wallpaper-state.json'
BG = '#11131a'
CARD = '#1e222d'
TEXT = '#eef0f6'
MUTED = '#a7acbb'
ACCENT = '#a0afff'
CURRENT_BACKGROUND = Path.home() / '.local/state/omarchy/current/background'

class Gallery(TkinterDnD.Tk):
    def __init__(self):
        super().__init__()
        self.title('Мои обои')
        self.geometry('960x660')
        self.minsize(620, 400)
        self.configure(bg=BG)
        self.images = []
        self.photos = []
        self.frames = {}
        self.current_wallpaper = None
        self.add_tile = None
        top = tk.Frame(self, bg=BG, padx=24, pady=18)
        top.pack(fill='x')
        tk.Label(top, text='Мои обои', bg=BG, fg=TEXT, font=('Sans', 20, 'bold')).pack(side='left')
        tk.Button(top, text='⤨  Перемешать', command=self.shuffle, bg=CARD, fg=TEXT,
                  activebackground='#343b4c', activeforeground=TEXT, relief='flat', padx=14, pady=7,
                  cursor='hand2').pack(side='right')
        self.timer = tk.Label(self, bg=BG, fg=TEXT, anchor='w', padx=25, font=('Sans', 11), text='Подключение к таймеру…')
        self.timer.pack(fill='x', pady=(0, 4))
        self.status = tk.Label(self, bg=BG, fg=MUTED, anchor='w', padx=25, text='Нажми на обои, чтобы установить')
        self.status.pack(fill='x')
        container = tk.Frame(self, bg=BG)
        container.pack(fill='both', expand=True, padx=18, pady=12)
        self.canvas = tk.Canvas(container, bg=BG, highlightthickness=0)
        scroll = tk.Scrollbar(container, orient='vertical', command=self.canvas.yview)
        self.canvas.configure(yscrollcommand=scroll.set)
        scroll.pack(side='right', fill='y')
        self.canvas.pack(side='left', fill='both', expand=True)
        self.grid_frame = tk.Frame(self.canvas, bg=BG)
        self.window = self.canvas.create_window((0, 0), window=self.grid_frame, anchor='nw')
        self.grid_frame.bind('<Configure>', lambda e: self.canvas.configure(scrollregion=self.canvas.bbox('all')))
        self.canvas.bind('<Configure>', self.resize)
        self.bind_all('<MouseWheel>', lambda e: self.canvas.yview_scroll(-int(e.delta / 120), 'units'))
        self.bind_all('<Button-4>', lambda e: self.canvas.yview_scroll(-1, 'units'))
        self.bind_all('<Button-5>', lambda e: self.canvas.yview_scroll(1, 'units'))
        self.columns = 0
        self.load()
        self.after(5000, self.refresh)
        self.update_timer()

    def update_timer(self):
        try:
            state = json.loads(STATE.read_text(encoding='utf-8'))
            pid = int(state['pid'])
            cmdline = Path(f'/proc/{pid}/cmdline').read_bytes()
            if b'wallpaper_rotation.py' not in cmdline:
                raise ValueError('Служба остановлена')
            if state['status'] == 'scheduled':
                seconds = max(0, math.ceil(float(state['deadline']) - time.monotonic()))
                hours, rem = divmod(seconds, 3600)
                minutes, seconds = divmod(rem, 60)
                value = f'{hours}:{minutes:02d}:{seconds:02d}' if hours else f'{minutes:02d}:{seconds:02d}'
                text = f'Следующая смена через {value}' if hours or minutes or seconds else 'Смена обоев…'
            elif state['status'] == 'empty':
                text = 'Ожидание картинок в очереди'
            else:
                text = 'Таймер недоступен: ошибка службы'
        except (OSError, ValueError, KeyError, TypeError):
            text = 'Таймер недоступен: служба остановлена'
        self.timer.config(text=text)
        self.update_current()
        self.after(1000, self.update_timer)

    def load(self):
        try:
            data = json.loads(CONFIG.read_text(encoding='utf-8'))
            folder = Path(data['folder']).expanduser()
            files = sorted(p.resolve() for p in folder.iterdir() if p.is_file() and p.suffix.lower() in {'.jpg', '.jpeg', '.png', '.webp', '.bmp'})
            if set(files) != set(self.images):
                self.images = files
                self.draw()
        except (OSError, ValueError, KeyError) as e:
            self.status.config(text=f'Не удалось прочитать папку: {e}')

    def refresh(self):
        self.load()
        self.after(5000, self.refresh)

    def resize(self, event):
        self.canvas.itemconfigure(self.window, width=event.width)
        columns = max(1, event.width // 285)
        if columns != self.columns:
            self.columns = columns
            self.draw()

    def update_current(self):
        try:
            current = CURRENT_BACKGROUND.resolve(strict=True)
        except OSError:
            current = None
        self.current_wallpaper = current
        for path, frame in self.frames.items():
            frame.configure(bg=ACCENT if path == current else CARD)

    def draw(self):
        for w in self.grid_frame.winfo_children(): w.destroy()
        self.photos = []
        self.frames = {}
        for i, path in enumerate(self.images):
            try:
                with Image.open(path) as img:
                    thumb = ImageOps.fit(img.convert('RGB'), (260, 146), Image.Resampling.LANCZOS)
                photo = ImageTk.PhotoImage(thumb)
                self.photos.append(photo)
                frame = tk.Frame(self.grid_frame, bg=CARD, padx=3, pady=3)
                button = tk.Button(frame, image=photo, bg=CARD, activebackground=ACCENT,
                                   relief='flat', borderwidth=0, highlightthickness=0, cursor='hand2',
                                   command=lambda p=path: self.apply(p))
                button.pack()
                frame.grid(row=i // max(1, self.columns), column=i % max(1, self.columns), padx=8, pady=8)
                self.frames[path] = frame
            except (OSError, ValueError):
                continue
        self.add_tile = tk.Frame(self.grid_frame, bg=CARD, width=266, height=152, cursor='hand2')
        self.add_tile.grid(row=len(self.images) // max(1, self.columns), column=len(self.images) % max(1, self.columns), padx=8, pady=8)
        self.add_tile.grid_propagate(False)
        self.add_surface = tk.Frame(self.add_tile, bg=CARD, width=260, height=146, cursor='hand2')
        self.add_surface.place(x=3, y=3, width=260, height=146)
        self.add_surface.bind('<Button-1>', lambda _event: self.choose_files())
        self.add_tile.bind('<Button-1>', lambda _event: self.choose_files())
        plus = tk.Label(self.add_surface, text='+', bg=CARD, fg=MUTED, font=('Sans', 38), cursor='hand2')
        plus.place(relx=.5, rely=.42, anchor='center')
        plus.bind('<Button-1>', lambda _event: self.choose_files())
        caption = tk.Label(self.add_surface, text='Добавить обои', bg=CARD, fg=MUTED, font=('Sans', 10), cursor='hand2')
        caption.place(relx=.5, rely=.82, anchor='center')
        caption.bind('<Button-1>', lambda _event: self.choose_files())
        for widget in (self.add_tile, self.add_surface, plus, caption):
            widget.drop_target_register(DND_FILES)
            widget.dnd_bind('<<Drop>>', self.drop_files)
            widget.bind('<Enter>', self.add_hover)
            widget.bind('<Leave>', self.add_leave)
        self.update_current()
        if not self.images:
            tk.Label(self.grid_frame, text='Пока здесь нет обоев', bg=BG, fg=MUTED).grid(padx=10, pady=20)

    def add_hover(self, _event=None):
        if self.add_tile and self.add_tile.winfo_exists():
            self.add_tile.configure(width=266, height=152, bg=ACCENT)

    def add_leave(self, _event=None):
        self.after(35, self.restore_add_tile_if_outside)

    def restore_add_tile_if_outside(self):
        if not self.add_tile or not self.add_tile.winfo_exists():
            return
        widget = self.winfo_containing(self.winfo_pointerx(), self.winfo_pointery())
        while widget is not None:
            if widget == self.add_tile:
                return
            widget = getattr(widget, 'master', None)
        self.add_tile.configure(width=266, height=152, bg=CARD)

    def choose_files(self):
        result = subprocess.run([
            'zenity', '--file-selection', '--multiple', '--separator=\n', '--modal',
            '--title=Добавить обои', '--file-filter=Изображения | *.jpg *.jpeg *.png *.webp *.bmp',
            '--file-filter=Все файлы | *'], capture_output=True, text=True)
        if result.returncode == 0 and result.stdout.strip():
            self.add_files(result.stdout.rstrip('\n').split('\n'))

    def drop_files(self, event):
        paths = self.tk.splitlist(event.data)
        self.add_files(paths)
        return 'break'

    def add_files(self, paths):
        data = json.loads(CONFIG.read_text(encoding='utf-8'))
        folder = Path(data['folder']).expanduser()
        folder.mkdir(parents=True, exist_ok=True)
        added, errors = 0, []
        for source_name in paths:
            source = Path(source_name)
            if source.suffix.lower() not in {'.jpg', '.jpeg', '.png', '.webp', '.bmp'}:
                errors.append(f'{source.name}: это не поддерживаемое изображение')
                continue
            try:
                with Image.open(source) as image:
                    image.verify()
                target = folder / source.name
                if source.resolve() == target.resolve():
                    continue
                if target.exists():
                    target = folder / f'{source.stem}-{int(time.time() * 1000)}{source.suffix.lower()}'
                temporary = target.with_name('.' + target.name + '.importing')
                shutil.copy2(source, temporary)
                temporary.replace(target)
                added += 1
            except (OSError, ValueError) as exc:
                errors.append(f'{source.name}: {exc}')
        self.load()
        if errors:
            messagebox.showwarning('Некоторые файлы не добавлены', '\n'.join(errors), parent=self)
        elif added:
            self.status.config(text=f'Добавлено обоев: {added}')

    def apply(self, path):
        try:
            subprocess.run(['omarchy', 'theme', 'bg', 'set', str(path)], check=True, capture_output=True, text=True)
            self.update_current()
            self.status.config(text='Обои установлены · следующая автоматическая смена по расписанию')
        except (OSError, subprocess.CalledProcessError) as e:
            messagebox.showerror('Не удалось установить обои', str(e), parent=self)

    def shuffle(self):
        result = subprocess.run(['systemctl', '--user', 'kill', '--kill-whom=main', '--signal=USR1', 'random-wallpaper.service'], capture_output=True, text=True)
        if result.returncode:
            messagebox.showerror('Не удалось перемешать очередь', result.stderr.strip(), parent=self)
            return
        random.shuffle(self.images)
        self.draw()
        self.status.config(text='Очередь перемешана · время следующей смены обновлено')

if __name__ == '__main__':
    Gallery().mainloop()
