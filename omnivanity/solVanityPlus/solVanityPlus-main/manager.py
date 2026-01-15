import subprocess
import sys
import time
import threading
import os
import itertools
import queue
import tkinter as tk
from tkinter import scrolledtext, messagebox, ttk

# Windows-specific input handling to clear buffer
try:
    import msvcrt
except ImportError:
    msvcrt = None

# ================= CONFIGURATION =================
MINER_SCRIPT = "main.py"
TARGETS_FILE = "vanity_targets.txt"

# PERFORMANCE SETTINGS
# BATCH_SIZE: 15 is safe for RTX cards
# ROTATE_INTERVAL: 60s ensures we cycle through your list
BATCH_SIZE = 15        
ROTATE_INTERVAL = 60   
MAX_CMD_LENGTH = 32000 

# FORCE GPU SELECTION
OS_ENV_VARS = {
    "CHOSEN_OPENCL_DEVICES": "0:0" 
}

# BASE COMMAND - NOW INCLUDES ITERATION BITS
BASE_COMMAND = [
    sys.executable, MINER_SCRIPT, "search-pubkey",
    "--select-device",
    "--count", "1000000000", 
    "--is-case-sensitive", "false",
    "--iteration-bits", "25"  # <--- THIS RESTORES YOUR SPEED
]

BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"

LEET_MAP = {
    'a': ['4'], 'b': ['8'], 'e': ['3'], 'g': ['9', '6'],
    'i': ['1'], 's': ['5', 'Z'], 't': ['7'], 'z': ['2'],
    'l': ['L', '1'], 'o': ['c', 'D']
}

MEME_NUMBERS = [
    "69", "369", "42c", "42D", "6", "9", "3", "7", "247"
]
# =================================================

class MinerManager:
    def __init__(self, log_callback):
        self.base_terms = set() 
        self.targets = set()
        
        self.process = None
        self.running = False
        self.lock = threading.Lock()
        self.current_batch_index = 0
        self.stop_rotation = False
        self.paused = False
        self.log = log_callback

    def is_valid_base58(self, s):
        for char in s:
            if char not in BASE58_ALPHABET: return False
        return True

    def clean_input(self, word):
        """
        Sanitizes input for Base58 compliance.
        INVALID: 0, O, I, l
        VALID: o, L, i, 1..9, A-Z, a-z
        """
        cleaned = ""
        for char in word:
            if char in BASE58_ALPHABET: 
                cleaned += char
            # Map INVALID chars to closest VALID lookalike
            elif char == 'l': cleaned += 'L'  # l -> L
            elif char == 'I': cleaned += 'i'  # I -> i
            elif char == '0': cleaned += 'o'  # 0 -> o (Zero to lower o)
            elif char == 'O': cleaned += 'o'  # O -> o (Big O to lower o)
            else: 
                pass # Drop other garbage like _ or -
        return cleaned

    def generate_fuzz_for_term(self, word):
        variations = set()
        base_word = self.clean_input(word)
        
        if not base_word: return set()

        if self.is_valid_base58(base_word):
            variations.add(base_word)

        if len(base_word) > 4:
            for i in [4, 5, 6]:
                if i < len(base_word):
                    prefix = base_word[:i]
                    if self.is_valid_base58(prefix):
                        variations.add(prefix)

        char_options = []
        for char in base_word:
            options = [char]
            lower_c = char.lower()
            if lower_c in LEET_MAP:
                for sub in LEET_MAP[lower_c]:
                    if sub in BASE58_ALPHABET: options.append(sub)
            char_options.append(list(set(options)))

        count = 0
        for p in itertools.product(*char_options):
            perm_str = "".join(p)
            if self.is_valid_base58(perm_str):
                variations.add(perm_str)
                count += 1
                if count >= 100: break
        
        number_variations = set()
        base_set = list(variations)[:20] 
        for v in base_set:
            for num in MEME_NUMBERS:
                cand1 = v + num
                if len(cand1) <= 7: number_variations.add(cand1)
                cand2 = num + v
                if len(cand2) <= 7: number_variations.add(cand2)
                if len(v) > 3:
                    mid = len(v) // 2
                    cand3 = v[:mid] + num + v[mid:]
                    if len(cand3) <= 7: number_variations.add(cand3)

        variations.update(number_variations)
        return variations

    def rebuild_active_pool(self):
        new_targets = set()
        for term in self.base_terms:
            if term.startswith("!"):
                # EXACT MATCH LOGIC
                raw_term = term[1:]
                clean = self.clean_input(raw_term)
                if clean: new_targets.add(clean)
            else:
                # FUZZ LOGIC
                fuzzed = self.generate_fuzz_for_term(term)
                new_targets.update(fuzzed)
        
        self.targets = new_targets
        self.log(f"[INFO] Pool Rebuilt. Base Terms: {len(self.base_terms)} -> Active Variations: {len(self.targets)}")

    def save_to_disk(self):
        try:
            with open(TARGETS_FILE, "w") as f:
                for t in sorted(self.base_terms):
                    f.write(f"{t}\n")
        except Exception as e:
            self.log(f"[!] Error saving targets: {e}")

    def load_from_disk(self):
        if not os.path.exists(TARGETS_FILE): return False
        try:
            loaded_count = 0
            with open(TARGETS_FILE, "r") as f:
                for line in f:
                    t = line.strip()
                    if t:
                        self.base_terms.add(t)
                        loaded_count += 1
            
            # FIX: Always return True if file exists, even if empty.
            # This prevents defaults from reloading if user cleared the list.
            if loaded_count > 0:
                self.log(f"[*] Restored {loaded_count} base terms from file.")
                self.rebuild_active_pool()
            else:
                self.log("[*] Target file is empty. Starting with 0 targets.")
                self.targets = set()
            
            return True
            
        except Exception as e:
            self.log(f"[!] Error loading targets: {e}")
        return False

    def load_defaults(self, initial_list):
        self.log(f"[*] First run. Loading defaults...")
        for t in initial_list:
            self.base_terms.add(t)
        self.rebuild_active_pool()
        self.save_to_disk()

    def add_target(self, base_term, fuzz=True):
        base_term = base_term.strip()
        if not base_term: return

        storage_term = base_term if fuzz else "!" + base_term

        if storage_term in self.base_terms:
            self.log(f"[!] '{base_term}' is already in your list.")
            return

        clean_check = self.clean_input(base_term)
        if not clean_check:
             self.log(f"[!] Warning: '{base_term}' contains no valid characters.")
             return

        self.base_terms.add(storage_term)
        self.rebuild_active_pool()
        self.save_to_disk()
        
        if fuzz:
            test_fuzz = self.generate_fuzz_for_term(base_term)
            preview = list(test_fuzz)[:3]
            self.log(f"[+] Added '{base_term}' + Fuzz. Vars: {', '.join(preview)}...")
        else:
            self.log(f"[+] Added EXACT match: '{clean_check}'")

    def remove_target(self, target):
        target = target.strip()
        exact_key = "!" + target
        
        removed = False
        if target in self.base_terms:
            self.base_terms.remove(target)
            removed = True
        if exact_key in self.base_terms:
            self.base_terms.remove(exact_key)
            removed = True
            
        if removed:
            self.log(f"[-] Removed '{target}' and all variants.")
            self.rebuild_active_pool()
            self.save_to_disk()
        else:
            self.log(f"[!] '{target}' not found in Base Terms list.")

    def clear_all_targets(self):
        self.base_terms.clear()
        self.targets.clear()
        self.save_to_disk()
        self.log("[-] CLEARED ALL TARGETS.")
        self.stop_miner()

    def start_rotation(self):
        t = threading.Thread(target=self._rotation_loop, daemon=True)
        t.start()

    def _rotation_loop(self):
        while not self.stop_rotation:
            if self.paused or not self.targets:
                time.sleep(1)
                continue

            all_targets = sorted(list(self.targets))
            total_targets = len(all_targets)
            
            if self.current_batch_index >= total_targets:
                self.current_batch_index = 0
            
            end_index = self.current_batch_index + BATCH_SIZE
            batch = all_targets[self.current_batch_index : end_index]
            self.current_batch_index = end_index if end_index < total_targets else 0

            self.run_batch(batch, self.current_batch_index, total_targets)
            
            for _ in range(ROTATE_INTERVAL):
                if self.stop_rotation: break
                if self.paused: 
                    self.stop_miner()
                    while self.paused and not self.stop_rotation:
                        time.sleep(0.5)
                    break
                time.sleep(1)

    def run_batch(self, batch, start_idx, total):
        self.stop_miner()
        cmd = BASE_COMMAND.copy()
        
        for t in batch:
            cmd.extend(["--starts-with", t])
            
        self.log(f"\n[*] ROTATING >> Mining Batch ({len(batch)} targets) [{start_idx}/{total}]")
        run_env = os.environ.copy()
        run_env.update(OS_ENV_VARS)

        try:
            with self.lock:
                creationflags = 0
                if sys.platform == "win32":
                    creationflags = subprocess.CREATE_NO_WINDOW
                
                self.process = subprocess.Popen(
                    cmd, 
                    env=run_env, 
                    creationflags=creationflags,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.STDOUT,
                    text=True,
                    bufsize=1
                )
                self.running = True
                
                t = threading.Thread(target=self._read_logs, args=(self.process,), daemon=True)
                t.start()
                
        except Exception as e:
            self.log(f"[!] Batch failed: {e}")

    def _read_logs(self, process):
        try:
            for line in iter(process.stdout.readline, ''):
                if not line: break
                clean_line = line.strip()
                if "BrokenPipeError" in clean_line or "WinError 232" in clean_line: continue
                if "Traceback" in clean_line or "multiprocessing" in clean_line: continue
                if clean_line: self.log(clean_line)
                if not self.running: break
        except: pass

    def stop_miner(self):
        with self.lock:
            if self.process:
                try:
                    pid = self.process.pid
                    subprocess.run(f"taskkill /F /T /PID {pid}", shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                except: pass
                self.process = None
                self.running = False

    def set_pause(self, paused):
        self.paused = paused
        if paused:
            self.stop_miner()
            self.log("\n[=] PAUSED. Mining stopped.")
        else:
            self.log("\n[=] RESUMING rotation...")

# ================= GUI CLASS =================

class VanityApp:
    def __init__(self, root):
        self.root = root
        self.root.title("Solana Vanity Manager (GUI)")
        self.root.geometry("900x600")
        
        self.log_queue = queue.Queue()
        self.manager = MinerManager(self.gui_log)
        
        self._setup_ui()
        self._start_manager()
        self._process_queue()

    def _setup_ui(self):
        control_frame = ttk.Frame(self.root, padding="10")
        control_frame.pack(fill=tk.X)

        # Inputs
        ttk.Label(control_frame, text="Search Term:").pack(side=tk.LEFT)
        self.entry_var = tk.StringVar()
        self.entry = ttk.Entry(control_frame, textvariable=self.entry_var, width=25)
        self.entry.pack(side=tk.LEFT, padx=5)
        self.entry.bind('<Return>', lambda e: self.add_target())

        # Fuzz Checkbox - DEFAULT OFF (As requested)
        self.fuzz_var = tk.BooleanVar(value=False)
        self.chk_fuzz = ttk.Checkbutton(control_frame, text="Auto-Fuzz", variable=self.fuzz_var)
        self.chk_fuzz.pack(side=tk.LEFT, padx=5)

        ttk.Button(control_frame, text="Add", command=self.add_target).pack(side=tk.LEFT, padx=2)
        ttk.Button(control_frame, text="Remove", command=self.remove_target).pack(side=tk.LEFT, padx=2)
        
        btn_clear = tk.Button(control_frame, text="CLEAR ALL", command=self.clear_all, fg="red", font=("Arial", 8, "bold"))
        btn_clear.pack(side=tk.LEFT, padx=15)

        self.btn_pause = ttk.Button(control_frame, text="Pause", command=self.toggle_pause)
        self.btn_pause.pack(side=tk.RIGHT, padx=5)

        # Stats
        stats_frame = ttk.Frame(self.root, padding="5")
        stats_frame.pack(fill=tk.X)
        self.lbl_stats = ttk.Label(stats_frame, text="Loading...")
        self.lbl_stats.pack(side=tk.LEFT)

        # Logs
        log_frame = ttk.Frame(self.root, padding="5")
        log_frame.pack(fill=tk.BOTH, expand=True)
        
        self.txt_log = scrolledtext.ScrolledText(log_frame, state='disabled', height=20, bg="black", fg="#00ff00", font=("Consolas", 10))
        self.txt_log.pack(fill=tk.BOTH, expand=True)

    def gui_log(self, message):
        self.log_queue.put(message)

    def _process_queue(self):
        try:
            while True:
                msg = self.log_queue.get_nowait()
                self.txt_log.configure(state='normal')
                self.txt_log.insert(tk.END, msg + "\n")
                self.txt_log.see(tk.END)
                self.txt_log.configure(state='disabled')
        except queue.Empty:
            pass
        
        status = "PAUSED" if self.manager.paused else "RUNNING"
        base_cnt = len(self.manager.base_terms)
        var_cnt = len(self.manager.targets)
        self.lbl_stats.config(text=f"Base Terms: {base_cnt} | Total Variations: {var_cnt} | Status: {status}")
        
        self.root.after(100, self._process_queue)

    def add_target(self):
        term = self.entry_var.get()
        fuzz_enabled = self.fuzz_var.get()
        
        if term:
            self.manager.add_target(term, fuzz=fuzz_enabled)
            self.entry_var.set("")

    def remove_target(self):
        term = self.entry_var.get()
        if term:
            self.manager.remove_target(term)
            self.entry_var.set("")

    def clear_all(self):
        if messagebox.askyesno("Confirm Clear", "Delete ALL search terms?"):
            self.manager.clear_all_targets()

    def toggle_pause(self):
        if self.manager.paused:
            self.manager.set_pause(False)
            self.btn_pause.config(text="Pause")
        else:
            self.manager.set_pause(True)
            self.btn_pause.config(text="Resume")

    def _start_manager(self):
        if not self.manager.load_from_disk():
            initial = ["ElonMusk", "CryptoRap", "Savage", "MemeGod"]
            self.manager.load_defaults(initial)
        self.manager.start_rotation()

    def on_close(self):
        self.manager.stop_rotation = True
        self.manager.stop_miner()
        self.root.destroy()
        os._exit(0)

if __name__ == "__main__":
    root = tk.Tk()
    app = VanityApp(root)
    root.protocol("WM_DELETE_WINDOW", app.on_close)
    root.mainloop()
