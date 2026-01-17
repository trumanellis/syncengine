# SyncEngine Design System

> **Aesthetic**: Terminal with depth
> **Principle**: Dark background, meaningful color, monospace clarity
> **Source**: syncengine-v0 reference implementation

---

## Design Philosophy

SyncEngine uses a **dark terminal aesthetic** — where a deep black background allows UI elements to stand out with purpose. The design prioritizes readability, clear hierarchy, and visual calm.

### Core Tenets

| Tenet | Expression |
|-------|------------|
| **Dark foundation** | Deep black backgrounds let key elements stand out |
| **Meaningful color** | Every color has semantic meaning (gold = important, cyan = interactive, green = status) |
| **Terminal clarity** | Monospace fonts and minimal chrome create focus |
| **Subtle depth** | Glows and shadows create layering without distraction |

---

## Color System

### Primary Palette

```css
:root {
  /* === BACKGROUNDS === */
  --void-black: #0a0a0a;           /* Primary background */
  --void-lighter: #0a0e0f;         /* Elevated surfaces */
  --void-border: #1a1a1a;          /* Subtle borders */

  /* === GREEN (Status, Success) === */
  --moss: #5a7a5a;                 /* Borders, inactive states */
  --moss-glow: #7cb87c;            /* Active status, success */
  --moss-bright: #39ff14;          /* High emphasis (rare) */

  /* === CYAN (Interactive, Links) === */
  --cyan: #00d4aa;                 /* Links, focus states, interactive elements */
  --cyan-glow: rgba(0, 212, 170, 0.3);

  /* === GOLD (Important, Headers) === */
  --gold: #d4af37;                 /* Headings, important labels */
  --gold-glow: rgba(212, 175, 55, 0.3);

  /* === TEXT === */
  --text-primary: #f5f5f5;         /* Primary text */
  --text-secondary: rgba(245, 245, 245, 0.7);  /* Secondary text */
  --text-muted: rgba(245, 245, 245, 0.5);      /* Placeholder, hints */

  /* === SEMANTIC === */
  --danger: #ff3366;               /* Errors, destructive actions */
  --warning: #ff9f00;              /* Warnings */
  --info: #5f8fff;                 /* Information */
  --lilac: #c4a7d7;                /* Accent */
}
```

### Color Semantics

| Color | Meaning | Usage |
|-------|---------|-------|
| **Gold** | Important, headers | Page titles, section headers |
| **Cyan** | Interactive, technical | Links, focused inputs, peer IDs |
| **Green** | Status, success | Online indicators, success states |
| **White** | Content | Body text, descriptions |

---

## Typography

### Font Stack

```css
/* Headlines — elegant serif */
--font-serif: 'Cormorant Garamond', Georgia, serif;

/* Body, technical, UI — monospace */
--font-mono: 'JetBrains Mono', 'SF Mono', 'Consolas', monospace;
```

**Font Loading:**
```html
<link href="https://fonts.googleapis.com/css2?family=Cormorant+Garamond:ital,wght@0,300;0,400;0,600;0,700;1,400&family=JetBrains+Mono:wght@300;400;500;600&display=swap" rel="stylesheet">
```

### Type Scale

| Token | Size | Use |
|-------|------|-----|
| `--text-xs` | 0.75rem (12px) | Hints, metadata |
| `--text-sm` | 0.875rem (14px) | Secondary text, labels |
| `--text-base` | 1rem (16px) | Body text |
| `--text-lg` | 1.125rem (18px) | Subtitles |
| `--text-xl` | 1.5rem (24px) | Section titles |
| `--text-2xl` | 2rem (32px) | Page subtitles |
| `--text-3xl` | 3rem (48px) | Page titles |

### Typography Patterns

```css
/* Page title — large, serif, gold, with glow */
.page-title {
  font-family: var(--font-serif);
  font-size: var(--text-3xl);
  font-weight: 400;
  color: var(--gold);
  text-shadow: 0 0 30px var(--gold-glow);
  letter-spacing: 0.1em;
}

/* Section header — serif, gold, italic */
.section-header {
  font-family: var(--font-serif);
  font-size: var(--text-xl);
  font-weight: 400;
  font-style: italic;
  color: var(--gold);
}

/* Body text — monospace, light */
.body-text {
  font-family: var(--font-mono);
  font-size: var(--text-base);
  font-weight: 400;
  color: var(--text-primary);
  line-height: 1.7;
}

/* Technical term — cyan */
.tech-term {
  color: var(--cyan);
}
```

---

## Components

### Status Indicator

```rust
#[component]
pub fn StatusIndicator(is_online: bool) -> Element {
    let label = if is_online { "online" } else { "offline" };

    rsx! {
        div { class: "status-indicator",
            span {
                class: if is_online { "status-dot online" } else { "status-dot" }
            }
            span { class: "status-label", "{label}" }
        }
    }
}
```

```css
.status-indicator {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--moss);
}

.status-dot.online {
  background: var(--moss-glow);
  box-shadow: 0 0 10px var(--moss-glow);
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}
```

### Category Pills

```css
.category-pills {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.pill {
  padding: 0.375rem 0.75rem;
  border: 1px solid var(--moss);
  border-radius: 4px;
  background: transparent;
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: all 0.2s ease;
}

.pill:hover {
  border-color: var(--moss-glow);
  color: var(--text-primary);
}

.pill.selected {
  background: var(--moss);
  border-color: var(--moss-glow);
  color: var(--text-primary);
}
```

### Input Fields

```css
.input-field {
  width: 100%;
  padding: 0.75rem 1rem;
  background: transparent;
  border: 1px solid var(--void-border);
  border-radius: 4px;
  color: var(--cyan);
  font-family: var(--font-mono);
  font-size: var(--text-base);
  transition: all 0.2s ease;
}

.input-field::placeholder {
  color: var(--text-muted);
  font-style: italic;
}

.input-field:focus {
  outline: none;
  border-color: var(--cyan);
  box-shadow: 0 0 0 1px var(--cyan), 0 0 20px var(--cyan-glow);
}

.input-label {
  display: block;
  margin-bottom: 0.5rem;
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
}
```

### Buttons

```css
/* Primary action button */
.btn-primary {
  padding: 0.75rem 2rem;
  background: transparent;
  border: 1px solid var(--moss);
  border-radius: 4px;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-base);
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn-primary:hover {
  border-color: var(--moss-glow);
  box-shadow: 0 0 20px rgba(124, 184, 124, 0.3);
  transform: translateY(-1px);
}

/* Destructive button */
.btn-danger {
  border-color: rgba(255, 51, 102, 0.3);
  color: var(--danger);
}

.btn-danger:hover {
  border-color: var(--danger);
  background: rgba(255, 51, 102, 0.1);
}
```

---

## Page Layouts

### Landing Page

```css
.landing {
  min-height: 100vh;
  background: var(--void-black);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 4rem 2rem;
}

.landing-header {
  text-align: center;
  max-width: 800px;
  margin-bottom: 4rem;
}

.tagline {
  font-family: var(--font-mono);
  font-size: var(--text-lg);
  color: var(--text-secondary);
  margin-top: 1rem;
}

.btn-enter {
  margin-top: 2rem;
  padding: 1rem 3rem;
  background: transparent;
  border: 1px solid var(--moss);
  border-radius: 4px;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-lg);
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn-enter:hover {
  border-color: var(--moss-glow);
  box-shadow: 0 0 30px rgba(124, 184, 124, 0.2);
}
```

### App Shell

```css
.app-shell {
  min-height: 100vh;
  background: var(--void-black);
  padding: 2rem;
}

.app-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 2rem;
}

.app-title {
  font-family: var(--font-serif);
  font-size: var(--text-2xl);
  font-weight: 400;
  color: var(--text-primary);
  letter-spacing: 0.05em;
}
```

---

## UI Terminology

Use clear, standard terminology:

| Term | Usage |
|------|-------|
| Online / Offline | Connection status |
| Create | New item creation |
| Task | Work item |
| Loading... | Loading state |
| Profile | User identity |
| Submit | Form submission |
| Cancel | Dismiss action |
| Delete | Remove item |
| Share | Share with others |
| Unpin | Stop pinning |

---

## Animation Guidelines

### Principles

1. **Subtle over dramatic** — Animations should feel natural, not attention-grabbing
2. **Glow over movement** — Prefer opacity/shadow changes over positional movement
3. **Reasonable duration** — 150ms-500ms for most transitions

### Standard Transitions

```css
--transition-fast: 150ms ease;
--transition-normal: 300ms ease;
--transition-slow: 500ms ease;

/* Pulse for active elements */
@keyframes pulse {
  0%, 100% { opacity: 1; box-shadow: 0 0 10px currentColor; }
  50% { opacity: 0.7; box-shadow: 0 0 20px currentColor; }
}

/* Fade for content */
@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}
```

---

## Accessibility

### Requirements

- Touch targets: minimum 44×44px
- Color contrast: WCAG AA (4.5:1 for text)
- Focus states: visible cyan outline
- Reduced motion: respect `prefers-reduced-motion`
- Screen reader: ARIA labels for status indicators

### Focus States

```css
*:focus-visible {
  outline: 2px solid var(--cyan);
  outline-offset: 2px;
}

@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

---

## Quick Reference

**Colors:**
- Background: `#0a0a0a`
- Gold (headers): `#d4af37`
- Cyan (interactive): `#00d4aa`
- Green (status): `#7cb87c`
- Text: `#f5f5f5`

**Fonts:**
- Titles: `Cormorant Garamond`
- Body: `JetBrains Mono`

---
