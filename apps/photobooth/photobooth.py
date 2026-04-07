"""
Amber Photobooth — NiceGUI frontend
"""

import asyncio
import html
import re
import shutil
from datetime import datetime
from pathlib import Path
from nicegui import ui, app

# ── Configuration ──────────────────────────────────────────────────────────────
IMAGE_OUTPUT = Path("image.png")
ARCHIVE_DIR = Path("capstone_images")
EMAIL_OUTPUT = Path("emails.txt")
COUNTDOWN_SECONDS = 3

LOGO_PATH = Path("assets/amber_colour.png")
POLEA_FONT_PATH = Path("assets/PoleaExtraBoldDemo-pg2x1.otf")

# QVGA = 320 x 240
QVGA_WIDTH = 320
QVGA_HEIGHT = 240

EMAIL_REGEX = re.compile(r"^[^@\s]+@[^@\s]+\.[^@\s]+$")

# ── Asset validation ───────────────────────────────────────────────────────────
logo_configured = bool(LOGO_PATH)
logo_exists = LOGO_PATH.exists() if logo_configured else False

font_configured = bool(POLEA_FONT_PATH)
font_exists = POLEA_FONT_PATH.exists() if font_configured else False

asset_errors = []
if not logo_configured:
    asset_errors.append("LOGO_PATH is not set")
elif not logo_exists:
    asset_errors.append(f"Logo file not found: {LOGO_PATH}")

if not font_configured:
    asset_errors.append("POLEA_FONT_PATH is not set")
elif not font_exists:
    asset_errors.append(f"Font file not found: {POLEA_FONT_PATH}")

assets_ready = not asset_errors

# Serve assets only if present
if logo_exists:
    app.add_static_file(local_file=str(LOGO_PATH), url_path="/logo")

if font_exists:
    app.add_static_file(
        local_file=str(POLEA_FONT_PATH),
        url_path="/fonts/PoleaExtraBoldDemo-pg2x1.otf",
    )

# ── State ──────────────────────────────────────────────────────────────────────
state = {
    "capturing": False,
    "email": "",
}

# ── Styles ─────────────────────────────────────────────────────────────────────
HEAD_HTML = f"""
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Poppins:wght@400;600;700;900&display=swap" rel="stylesheet">

<style>
  {"@font-face { font-family: 'Polea'; src: url('/fonts/PoleaExtraBoldDemo-pg2x1.otf') format('opentype'); font-style: normal; font-weight: normal; }" if font_exists else ""}

  *, *::before, *::after {{
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }}

  html, body, .nicegui-content {{
    width: 100%;
    min-height: 100%;
  }}

  body, .nicegui-content {{
    background: #FFFFFF !important;
    overflow: auto;
  }}

  body::before {{
    content: "";
    position: fixed;
    inset: 0;
    background-image:
      linear-gradient(rgba(255,177,0,0.03) 1px, transparent 1px),
      linear-gradient(90deg, rgba(255,177,0,0.03) 1px, transparent 1px);
    background-size: 40px 40px;
    pointer-events: none;
    z-index: 0;
  }}

  #pb-root {{
    position: relative;
    z-index: 1;
    min-height: 100vh;
    display: grid;
    place-items: center;
    padding: clamp(12px, 2vw, 24px);
  }}

  .pb-shell {{
    width: min(92vw, 900px);
    max-width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: clamp(10px, 1.6vh, 18px);
  }}

  .pb-header {{
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: clamp(0.8rem, 2vw, 1.2rem);
    text-align: center;
    user-select: none;
    flex-wrap: wrap;
  }}

  .pb-logo-img {{
    height: clamp(40px, 5vw, 56px);
    width: auto;
    object-fit: contain;
    flex-shrink: 0;
  }}

  .pb-asset-error-badge {{
    min-height: clamp(40px, 5vw, 56px);
    min-width: clamp(40px, 5vw, 56px);
    padding: 0.5rem 0.7rem;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid #cc3300;
    background: #fff4f2;
    color: #cc3300;
    font-family: 'Poppins', sans-serif;
    font-size: clamp(0.58rem, 1.4vw, 0.7rem);
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    text-align: center;
  }}

  .pb-title-block {{
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.15rem;
    min-width: 0;
  }}

  .pb-title {{
    font-family: {'"Polea", "Poppins", sans-serif' if font_exists else '"Poppins", sans-serif'};
    font-size: clamp(1.7rem, 4vw, 3.8rem);
    color: #FFB100;
    letter-spacing: 0;
    line-height: 1;
    text-align: center;
    word-break: break-word;
  }}

  .pb-title-dot {{
    color: #FF7701;
  }}

  .pb-subtitle {{
    font-family: 'Poppins', sans-serif;
    font-weight: 400;
    font-size: clamp(0.7rem, 1.5vw, 0.85rem);
    letter-spacing: 0.22em;
    text-transform: uppercase;
    color: #3a3a3a;
    text-align: center;
  }}

  /* True QVGA pane: height is always width * 240 / 320 */
  .pb-pane-wrap {{
    --pane-width: min(
      88vw,
      720px,
      calc((100vh - 270px) * {QVGA_WIDTH} / {QVGA_HEIGHT})
    );
    position: relative;
    width: var(--pane-width);
    height: calc(var(--pane-width) * {QVGA_HEIGHT} / {QVGA_WIDTH});
    flex: 0 0 auto;
  }}

  .pb-corner {{
    position: absolute;
    width: 24px;
    height: 24px;
    border-color: #FFB100;
    border-style: solid;
    z-index: 3;
    pointer-events: none;
  }}
  .pb-corner.tl {{ top: -5px; left: -5px; border-width: 2px 0 0 2px; }}
  .pb-corner.tr {{ top: -5px; right: -5px; border-width: 2px 2px 0 0; }}
  .pb-corner.bl {{ bottom: -5px; left: -5px; border-width: 0 0 2px 2px; }}
  .pb-corner.br {{ bottom: -5px; right: -5px; border-width: 0 2px 2px 0; }}

  .pb-pane-inner {{
    position: absolute;
    inset: 0;
    border: 1px solid #1c1c1c;
    border-radius: 2px;
    overflow: hidden;
    background: #050505;
    box-shadow:
      0 0 0 1px #111,
      0 20px 60px rgba(0,0,0,0.9),
      0 0 80px rgba(255,177,0,0.04);
  }}

  #captured-img {{
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
    opacity: 0;
    transition: opacity 0.5s ease;
    position: absolute;
    inset: 0;
  }}
  #captured-img.visible {{
    opacity: 1;
  }}

  #prompt-overlay {{
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: clamp(12px, 2vw, 24px);
    gap: clamp(0.7rem, 1.8vh, 1.2rem);
    background: #050505;
    z-index: 2;
    transition: opacity 0.35s ease;
  }}
  #prompt-overlay.hidden {{
    opacity: 0;
    pointer-events: none;
  }}

  #idle-hint {{
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: clamp(0.7rem, 1.5vh, 1rem);
    text-align: center;
    width: min(100%, 520px);
    max-width: 100%;
    min-width: 0;
  }}

  .pb-idle-line {{
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.6rem;
    flex-wrap: wrap;
    max-width: 100%;
  }}

  .pb-idle-text {{
    font-family: 'Poppins', sans-serif;
    font-weight: 400;
    font-size: clamp(0.68rem, 1.3vw, 0.82rem);
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: #2e2e2e;
    text-align: center;
  }}

  .pb-kbd {{
    font-family: 'Poppins', sans-serif;
    font-weight: 600;
    font-size: clamp(0.7rem, 1.4vw, 0.82rem);
    letter-spacing: 0.12em;
    color: #FFB100;
    border: 1.5px solid #2a2200;
    border-radius: 4px;
    padding: 0.3em 0.9em;
    background: rgba(255,177,0,0.05);
    box-shadow: 0 3px 0 #1a1500;
    flex-shrink: 0;
  }}

  .pb-idle-error-title {{
    font-family: 'Poppins', sans-serif;
    font-weight: 700;
    font-size: clamp(0.82rem, 1.6vw, 0.95rem);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: #cc3300;
    text-align: center;
  }}

  .pb-idle-error-body {{
    font-family: 'Poppins', sans-serif;
    font-weight: 400;
    font-size: clamp(0.72rem, 1.25vw, 0.82rem);
    line-height: 1.5;
    color: #d0d0d0;
    letter-spacing: 0.01em;
    text-align: center;
    overflow-wrap: anywhere;
  }}

  .pb-idle-error-code {{
    width: min(100%, 440px);
    font-family: monospace;
    font-size: clamp(0.68rem, 1.1vw, 0.78rem);
    color: #ffb4a8;
    background: rgba(204, 51, 0, 0.12);
    border: 1px solid rgba(204, 51, 0, 0.35);
    padding: 0.7rem 0.9rem;
    border-radius: 6px;
    white-space: pre-wrap;
    text-align: left;
    overflow-wrap: anywhere;
  }}

  .pb-email-wrap {{
    width: min(88vw, 720px);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.45rem;
    text-align: center;
  }}

  .pb-email-label {{
    width: 100%;
    font-family: 'Poppins', sans-serif;
    font-size: clamp(0.62rem, 1.2vw, 0.72rem);
    font-weight: 600;
    letter-spacing: 0.24em;
    text-transform: uppercase;
    color: #5a5a5a;
    text-align: center;
  }}

  .pb-email-box {{
    width: 100%;
    min-height: 52px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.85rem 1rem;
    border: 1px solid #d7d7d7;
    background: #ffffff;
    box-shadow: 0 8px 24px rgba(0,0,0,0.06);
    position: relative;
    overflow: hidden;
  }}

  .pb-email-value {{
    width: 100%;
    font-family: 'Poppins', sans-serif;
    font-size: clamp(0.92rem, 2vw, 1rem);
    font-weight: 500;
    color: #222222;
    text-align: center;
    word-break: break-word;
    overflow-wrap: anywhere;
    white-space: normal;
  }}

  .pb-email-placeholder {{
    font-family: 'Poppins', sans-serif;
    font-size: clamp(0.92rem, 2vw, 1rem);
    font-weight: 400;
    color: #9a9a9a;
  }}

  .pb-email-caret {{
    display: inline-block;
    width: 1px;
    height: 1.1em;
    background: #FFB100;
    margin-left: 3px;
    vertical-align: -0.15em;
    animation: pbBlink 1s steps(1) infinite;
  }}

  @keyframes pbBlink {{
    50% {{ opacity: 0; }}
  }}

  .pb-email-help {{
    width: 100%;
    font-family: 'Poppins', sans-serif;
    font-size: clamp(0.62rem, 1.15vw, 0.7rem);
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: #787878;
    text-align: center;
    line-height: 1.5;
    overflow-wrap: anywhere;
  }}

  .pb-statusbar {{
    width: min(88vw, 720px);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.6rem;
    min-height: 1.5em;
    text-align: center;
  }}

  .pb-status-dot {{
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: #222;
    flex-shrink: 0;
    transition: background 0.3s ease;
  }}

  .pb-status {{
    font-family: 'Poppins', sans-serif;
    font-weight: 400;
    font-size: clamp(0.58rem, 1vw, 0.66rem);
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: #252525;
    transition: color 0.3s ease;
    text-align: center;
    line-height: 1.5;
    overflow-wrap: anywhere;
  }}

  .pb-statusbar.active  .pb-status-dot {{ background: #FFB100; box-shadow: 0 0 6px #FFB100; }}
  .pb-statusbar.active  .pb-status     {{ color: #FFB100; }}
  .pb-statusbar.success .pb-status-dot {{ background: #FF7701; }}
  .pb-statusbar.success .pb-status     {{ color: #FF7701; }}
  .pb-statusbar.error   .pb-status-dot {{ background: #cc3300; }}
  .pb-statusbar.error   .pb-status     {{ color: #cc3300; }}

  #countdown-num {{
    font-family: 'Poppins', sans-serif;
    font-weight: 900;
    font-size: clamp(4.2rem, 14vw, 11rem);
    color: #FFB100;
    text-shadow:
      0 0 40px rgba(255,177,0,0.6),
      0 0 100px rgba(255,119,1,0.25);
    line-height: 1;
    display: none;
    text-align: center;
    animation: popIn 0.35s cubic-bezier(0.34,1.56,0.64,1);
  }}

  @keyframes popIn {{
    from {{ transform: scale(2); opacity: 0; }}
    to   {{ transform: scale(1); opacity: 1; }}
  }}

  #flash {{
    position: absolute;
    inset: 0;
    background: #fff8e0;
    opacity: 0;
    z-index: 10;
    pointer-events: none;
    transition: opacity 0.04s ease;
  }}
  #flash.on {{
    opacity: 1;
  }}

  .pb-loading {{
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 6px;
    background: rgba(255,255,255,0.08);
    overflow: hidden;
    z-index: 12;
    opacity: 0;
    transition: opacity 0.2s ease;
    pointer-events: none;
  }}

  .pb-loading.visible {{
    opacity: 1;
  }}

  .pb-loading-bar {{
    position: absolute;
    top: 0;
    bottom: 0;
    width: 32%;
    background: linear-gradient(90deg, #FFB100, #FF7701);
    box-shadow: 0 0 10px rgba(255,177,0,0.35);
    animation: pb-loading-slide 1.15s ease-in-out infinite;
  }}

  @keyframes pb-loading-slide {{
    0%   {{ left: -35%; }}
    100% {{ left: 100%; }}
  }}

  @keyframes shimmer {{
    from {{ background-position: -200% 0; }}
    to   {{ background-position: 200% 0; }}
  }}
  .pb-shimmer::after {{
    content: "";
    position: absolute;
    inset: 0;
    background: linear-gradient(
      100deg,
      transparent 30%,
      rgba(255,177,0,0.07) 50%,
      transparent 70%
    );
    background-size: 200% 100%;
    animation: shimmer 1.2s ease forwards;
    pointer-events: none;
    z-index: 3;
  }}

  @media (max-width: 560px) {{
    .pb-header {{
      flex-direction: column;
    }}

    .pb-subtitle {{
      letter-spacing: 0.16em;
    }}

    .pb-corner {{
      width: 18px;
      height: 18px;
    }}

    .pb-idle-line {{
      gap: 0.35rem;
    }}
  }}

  @media (max-height: 740px) {{
    .pb-pane-wrap {{
      --pane-width: min(
        88vw,
        600px,
        calc((100vh - 235px) * {QVGA_WIDTH} / {QVGA_HEIGHT})
      );
    }}

    .pb-email-box {{
      min-height: 46px;
      padding: 0.7rem 0.85rem;
    }}
  }}
</style>
"""

# ── JavaScript ─────────────────────────────────────────────────────────────────
INIT_JS = """
document.addEventListener('keydown', function(e) {
    if (e.repeat) return;

    if (e.code === 'Space') {
        e.preventDefault();
        emitEvent('space_pressed');
        return;
    }

    if (e.key === 'Backspace') {
        e.preventDefault();
        emitEvent('email_backspace');
        return;
    }

    if (e.key === 'Enter') {
        e.preventDefault();
        emitEvent('email_submit');
        return;
    }

    if (e.key === 'Escape') {
        e.preventDefault();
        emitEvent('email_reset');
        return;
    }

    const printable = e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey;
    if (printable) {
        e.preventDefault();
        emitEvent('email_key', {key: e.key});
    }
});

window.runCountdown = async function(seconds) {
    const idleEl  = document.getElementById('idle-hint');
    const countEl = document.getElementById('countdown-num');
    const flash   = document.getElementById('flash');

    if (idleEl) idleEl.style.display = 'none';
    if (countEl) countEl.style.display = 'block';

    for (let i = seconds; i >= 1; i--) {
        if (countEl) {
            countEl.textContent = i;
            countEl.style.animation = 'none';
            countEl.offsetHeight;
            countEl.style.animation = 'popIn 0.35s cubic-bezier(0.34,1.56,0.64,1)';
        }
        await new Promise(r => setTimeout(r, 1000));
    }

    flash.classList.add('on');
    await new Promise(r => setTimeout(r, 80));
    flash.classList.remove('on');

    if (countEl) countEl.style.display = 'none';
    emitEvent('shutter_fired');
};

window.showCapturedImage = function(url) {
    const img     = document.getElementById('captured-img');
    const overlay = document.getElementById('prompt-overlay');
    const pane    = document.querySelector('.pb-pane-inner');
    const loading = document.getElementById('pb-loading');

    img.src = url;
    img.classList.add('visible');
    overlay.classList.add('hidden');
    if (loading) loading.classList.remove('visible');

    if (pane) {
        pane.classList.add('pb-shimmer');
        setTimeout(() => pane.classList.remove('pb-shimmer'), 1400);
    }
};

window.showLoadingBar = function() {
    const el = document.getElementById('pb-loading');
    if (el) el.classList.add('visible');
};

window.hideLoadingBar = function() {
    const el = document.getElementById('pb-loading');
    if (el) el.classList.remove('visible');
};

window.resetToIdle = function() {
    const idleEl  = document.getElementById('idle-hint');
    const countEl = document.getElementById('countdown-num');
    const overlay = document.getElementById('prompt-overlay');
    const loading = document.getElementById('pb-loading');

    if (idleEl) idleEl.style.display = '';
    if (countEl) {
        countEl.style.display = 'none';
        countEl.textContent = '';
    }
    if (overlay) overlay.classList.remove('hidden');
    if (loading) loading.classList.remove('visible');
};
"""

def build_idle_content() -> str:
    if assets_ready:
        return """
        <div class="pb-idle-line">
          <span class="pb-idle-text">press</span>
          <span class="pb-kbd">SPACE</span>
          <span class="pb-idle-text">to capture</span>
        </div>
        """

    error_lines = "\\n".join(asset_errors)
    return f"""
    <div class="pb-idle-error-title">Asset configuration error</div>
    <div class="pb-idle-error-body">
      Required UI assets are missing or not configured correctly.
      Update the file paths below before using the photobooth.
    </div>
    <div class="pb-idle-error-code">{html.escape(error_lines)}</div>
    """

def render_email_value(email: str) -> str:
    if email:
        return f'{html.escape(email)}<span class="pb-email-caret"></span>'
    return '<span class="pb-email-placeholder">type email address<span class="pb-email-caret"></span></span>'

def is_valid_email(email: str) -> bool:
    return bool(EMAIL_REGEX.fullmatch(email.strip()))

def append_email_to_file(email: str) -> None:
    EMAIL_OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    with EMAIL_OUTPUT.open("a", encoding="utf-8") as f:
        f.write(email.strip() + "\\n")

# ── Page ───────────────────────────────────────────────────────────────────────
@ui.page("/")
def photobooth_page():
    ui.add_head_html(HEAD_HTML, shared=True)
    ui.run_javascript(INIT_JS)

    with ui.element("div").props('id="pb-root"'):
        with ui.element("div").classes("pb-shell"):

            with ui.element("div").classes("pb-header"):
                if logo_exists:
                    ui.html('<img class="pb-logo-img" src="/logo" alt="Amber logo">')
                else:
                    ui.html('<div class="pb-asset-error-badge">NO LOGO</div>')

                with ui.element("div").classes("pb-title-block"):
                    ui.html('<span class="pb-title">amber<span class="pb-title-dot">.</span></span>')
                    ui.label("photobooth").classes("pb-subtitle")

            with ui.element("div").classes("pb-pane-wrap"):
                for cls in ("tl", "tr", "bl", "br"):
                    ui.element("div").classes(f"pb-corner {cls}")

                with ui.element("div").classes("pb-pane-inner"):
                    ui.html('<img id="captured-img" alt="Captured photo">')

                    ui.html(f"""
                    <div id="prompt-overlay">
                      <div id="idle-hint">
                        {build_idle_content()}
                      </div>
                      <span id="countdown-num"></span>
                    </div>
                    """)

                    ui.html('<div id="flash"></div>')

                    ui.html("""
                    <div id="pb-loading" class="pb-loading">
                      <div class="pb-loading-bar"></div>
                    </div>
                    """)

            with ui.element("div").classes("pb-email-wrap"):
                ui.label("email").classes("pb-email-label")
                with ui.element("div").classes("pb-email-box"):
                    email_display = ui.html(render_email_value(state["email"])).classes("pb-email-value")
                ui.label(
                    "type to enter · enter to save · esc to clear · space to capture"
                ).classes("pb-email-help")

            with ui.element("div").classes("pb-statusbar") as statusbar:
                ui.element("div").classes("pb-status-dot")
                status = ui.label(
                    "waiting" if assets_ready else "asset configuration error"
                ).classes("pb-status")
                if not assets_ready:
                    statusbar.classes(add="error")

    def refresh_email_display() -> None:
        email_display.set_content(render_email_value(state["email"]))

    async def on_email_key(e):
        if state["capturing"]:
            return
        key = e.args.get("key", "")
        if not key:
            return
        state["email"] += key
        refresh_email_display()

    async def on_email_backspace():
        if state["capturing"]:
            return
        if state["email"]:
            state["email"] = state["email"][:-1]
            refresh_email_display()

    async def on_email_reset():
        if state["capturing"]:
            return
        state["email"] = ""
        refresh_email_display()
        await ui.run_javascript("resetToIdle()")
        statusbar.classes(remove="active success error")
        status.set_text("waiting")

    async def on_email_submit():
        if state["capturing"]:
            return

        email = state["email"].strip()
        if not email:
            statusbar.classes(remove="active success", add="error")
            status.set_text("enter an email first")
            return

        if not is_valid_email(email):
            statusbar.classes(remove="active success", add="error")
            status.set_text("invalid email")
            return

        try:
            append_email_to_file(email)
            state["email"] = ""
            refresh_email_display()
            statusbar.classes(remove="active error", add="success")
            status.set_text(f"email saved  ·  {EMAIL_OUTPUT}")
        except Exception as exc:
            statusbar.classes(remove="active success", add="error")
            status.set_text(f"email save error  ·  {exc}")

    async def on_space_pressed():
        if not assets_ready:
            statusbar.classes(remove="active success", add="error")
            status.set_text("asset configuration error")
            return

        if state["capturing"]:
            return

        state["capturing"] = True
        statusbar.classes(remove="error success", add="active")
        status.set_text("get ready")
        await ui.run_javascript(f"runCountdown({COUNTDOWN_SECONDS})")

    async def on_shutter_fired():
        statusbar.classes(remove="error success", add="active")
        status.set_text("capturing")
        await ui.run_javascript("showLoadingBar()")

        try:
            proc = await asyncio.create_subprocess_exec(
                "./capture",
                "camera",
                "image",
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            stdout, stderr = await proc.communicate()

            if proc.returncode != 0:
                err = stderr.decode(errors="replace").strip().splitlines()
                raise RuntimeError(err[-1] if err else f"exit code {proc.returncode}")

            if not IMAGE_OUTPUT.exists():
                raise RuntimeError(f"{IMAGE_OUTPUT} not found after capture")

            ARCHIVE_DIR.mkdir(parents=True, exist_ok=True)
            timestamp = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
            archive_path = ARCHIVE_DIR / f"{timestamp}.png"
            shutil.copy2(IMAGE_OUTPUT, archive_path)

            app.add_static_file(
                local_file=str(IMAGE_OUTPUT),
                url_path="/current_capture.png",
            )
            cache_bust = datetime.now().strftime("%H%M%S%f")
            await ui.run_javascript(
                f"showCapturedImage('/current_capture.png?v={cache_bust}')"
            )
            await ui.run_javascript("hideLoadingBar()")

            statusbar.classes(remove="active error", add="success")
            status.set_text(f"saved  ·  capstone_images/{timestamp}.png")

        except Exception as exc:
            await ui.run_javascript("hideLoadingBar()")
            statusbar.classes(remove="active success", add="error")
            status.set_text(f"error  ·  {exc}")
            await ui.run_javascript("resetToIdle()")

        finally:
            state["capturing"] = False

    ui.on("email_key", on_email_key)
    ui.on("email_backspace", on_email_backspace)
    ui.on("email_submit", on_email_submit)
    ui.on("email_reset", on_email_reset)
    ui.on("space_pressed", on_space_pressed)
    ui.on("shutter_fired", on_shutter_fired)

if __name__ in {"__main__", "__mp_main__"}:
    ui.run(
        title="Amber Photobooth",
        host="0.0.0.0",
        port=3000,
        reload=False,
        dark=True,
        favicon="🦋",
    )