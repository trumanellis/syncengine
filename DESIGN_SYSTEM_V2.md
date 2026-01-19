# SyncEngine Design System v2 — Minimal Terminal

> **Aesthetic**: Minimal terminal
> **Principle**: Calm, restrained, precise. The interface recedes to let content lead.
> **Previous version**: `archive/DESIGN_SYSTEM_V1.md`

---

## Philosophy

The interface should feel like a well-designed command-line tool: focused, efficient, and unobtrusive. Every element earns its place. Decoration without purpose is removed.

### Design Principles

| Principle | Expression |
|-----------|------------|
| **One action per screen** | Clear primary action, minimal competing elements |
| **Generous whitespace** | Space creates hierarchy better than visual weight |
| **Three type sizes** | Small, base, large — nothing more |
| **4pt/8pt spacing** | Consistent rhythm throughout |
| **Subtle motion** | 150-200ms transitions only |
| **No decorative animation** | If it doesn't convey information, remove it |

---

## Colors

Colors are preserved from v1. Every color carries semantic meaning.

```css
:root {
  /* === BACKGROUNDS === */
  --void-black: #0a0a0a;           /* Primary background */
  --void-lighter: #0a0e0f;         /* Elevated surfaces */
  --void-border: #1a1a1a;          /* Subtle borders */

  /* === MOSS (Status, Success) === */
  --moss: #5a7a5a;                 /* Borders, inactive states */
  --moss-glow: #7cb87c;            /* Active status, success */

  /* === CYAN (Interactive) === */
  --cyan: #00d4aa;                 /* Links, focus states, interactive */

  /* === GOLD (Important) === */
  --gold: #d4af37;                 /* Headings, important labels */

  /* === TEXT === */
  --text-primary: #f5f5f5;         /* Primary text */
  --text-secondary: rgba(245, 245, 245, 0.7);  /* Secondary text */
  --text-muted: rgba(245, 245, 245, 0.5);      /* Placeholder, hints */

  /* === SEMANTIC === */
  --danger: #ff3366;               /* Errors, destructive actions */
}
```

### Color Usage

| Color | Meaning | Usage |
|-------|---------|-------|
| **Gold** | Important | Page titles, section headers |
| **Cyan** | Interactive | Links, focused inputs, peer IDs |
| **Moss** | Status | Online indicators, primary buttons |
| **Danger** | Destructive | Delete actions, errors |

---

## Typography

### Font Stack

```css
/* Headlines — elegant serif */
--font-serif: 'Cormorant Garamond', Georgia, serif;

/* Body, UI — monospace */
--font-mono: 'JetBrains Mono', 'SF Mono', 'Consolas', monospace;
```

**Font Loading:**
```html
<link href="https://fonts.googleapis.com/css2?family=Cormorant+Garamond:wght@400;600&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
```

### Type Scale (3 sizes only)

```css
--text-sm: 0.875rem;   /* 14px — labels, metadata, hints */
--text-base: 1rem;     /* 16px — body, inputs, buttons */
--text-lg: 1.5rem;     /* 24px — titles, headers */
```

| Token | Size | Use |
|-------|------|-----|
| `--text-sm` | 14px | Labels, metadata, secondary text |
| `--text-base` | 16px | Body text, inputs, buttons |
| `--text-lg` | 24px | Page titles, section headers |

### Typography Patterns

```css
/* Page title — serif, gold */
.page-title {
  font-family: var(--font-serif);
  font-size: var(--text-lg);
  font-weight: 400;
  color: var(--gold);
  letter-spacing: 0.05em;
}

/* Section header — serif, gold, italic */
.section-header {
  font-family: var(--font-serif);
  font-size: var(--text-lg);
  font-weight: 400;
  font-style: italic;
  color: var(--gold);
}

/* Body text — monospace */
.body-text {
  font-family: var(--font-mono);
  font-size: var(--text-base);
  font-weight: 400;
  color: var(--text-primary);
  line-height: 1.6;
}

/* Label — monospace, small */
.label {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-secondary);
}
```

---

## Spacing

Strict 4pt grid. All spacing values are multiples of 4px.

```css
--space-1: 4px;
--space-2: 8px;
--space-3: 12px;
--space-4: 16px;
--space-6: 24px;
--space-8: 32px;
```

### Usage Guidelines

| Token | Use |
|-------|-----|
| `--space-1` | Tight gaps (icon + label) |
| `--space-2` | Default gap between related elements |
| `--space-3` | Padding inside small components |
| `--space-4` | Padding inside cards, section gaps |
| `--space-6` | Major section spacing |
| `--space-8` | Page-level margins |

---

## Motion

Two transition speeds only. All animations serve a purpose.

```css
--transition-fast: 150ms ease;    /* Micro-interactions (hover, focus) */
--transition-normal: 200ms ease;  /* State changes (expand, modal) */
```

### Allowed Animations

| Animation | Use | Duration |
|-----------|-----|----------|
| `fadeIn` | Content appearing | 200ms |
| `spin` | Loading spinner | 1s linear |
| `modal-appear` | Modal entrance | 200ms |

```css
@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@keyframes modal-appear {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}
```

### Removed (from v1)

Do not use:
- `pulse`, `breathe`, `sacred-pulse`
- `listening-glow`, `resonating-pulse`
- `float`, `ring-pulse`, `seeking-pulse`
- Any animation with duration > 500ms
- Any animation that loops indefinitely (except loading spinners)

---

## Components

### Buttons (3 variants)

```css
/* Base button styles */
.btn {
  padding: var(--space-3) var(--space-4);
  background: transparent;
  border: 1px solid;
  border-radius: 4px;
  font-family: var(--font-mono);
  font-size: var(--text-base);
  cursor: pointer;
  transition: border-color var(--transition-fast),
              background var(--transition-fast);
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Primary — moss border */
.btn-primary {
  border-color: var(--moss);
  color: var(--text-primary);
}

.btn-primary:hover:not(:disabled) {
  border-color: var(--moss-glow);
  background: rgba(124, 184, 124, 0.1);
}

/* Secondary — void border */
.btn-secondary {
  border-color: var(--void-border);
  color: var(--text-secondary);
}

.btn-secondary:hover:not(:disabled) {
  border-color: var(--text-muted);
  color: var(--text-primary);
}

/* Destructive — danger */
.btn-destructive {
  border-color: rgba(255, 51, 102, 0.5);
  color: var(--danger);
}

.btn-destructive:hover:not(:disabled) {
  border-color: var(--danger);
  background: rgba(255, 51, 102, 0.1);
}
```

### Input Fields

```css
.input {
  width: 100%;
  padding: var(--space-3) var(--space-4);
  background: transparent;
  border: 1px solid var(--void-border);
  border-radius: 4px;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-base);
  transition: border-color var(--transition-fast);
}

.input::placeholder {
  color: var(--text-muted);
}

.input:focus {
  outline: none;
  border-color: var(--cyan);
  box-shadow: 0 0 0 1px var(--cyan);
}

.input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.input-label {
  display: block;
  margin-bottom: var(--space-2);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-secondary);
}
```

### Status Indicator (Simple Dot)

```css
.status-indicator {
  display: flex;
  align-items: center;
  gap: var(--space-2);
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
}

.status-dot.connecting {
  background: var(--gold);
}

.status-dot.error {
  background: var(--danger);
}
```

```rust
#[component]
pub fn StatusIndicator(status: ConnectionStatus) -> Element {
    let (class, label) = match status {
        ConnectionStatus::Online => ("status-dot online", "online"),
        ConnectionStatus::Connecting => ("status-dot connecting", "connecting"),
        ConnectionStatus::Offline => ("status-dot", "offline"),
        ConnectionStatus::Error => ("status-dot error", "error"),
    };

    rsx! {
        div { class: "status-indicator",
            span { class: "{class}" }
            span { "{label}" }
        }
    }
}
```

### Cards

```css
.card {
  padding: var(--space-4);
  background: var(--void-lighter);
  border: 1px solid var(--void-border);
  border-radius: 4px;
}

.card-title {
  font-family: var(--font-serif);
  font-size: var(--text-lg);
  font-style: italic;
  color: var(--gold);
  margin-bottom: var(--space-3);
}
```

### Modal

```css
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  animation: fadeIn var(--transition-normal);
}

.modal {
  max-width: 480px;
  width: 90%;
  padding: var(--space-6);
  background: var(--void-black);
  border: 1px solid var(--gold);
  border-radius: 4px;
  animation: modal-appear var(--transition-normal);
}

.modal-title {
  font-family: var(--font-serif);
  font-size: var(--text-lg);
  font-style: italic;
  color: var(--gold);
  margin-bottom: var(--space-4);
}
```

---

## States

### Loading

```css
.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-8);
}

.loading-spinner {
  width: 24px;
  height: 24px;
  border: 2px solid var(--void-border);
  border-top-color: var(--moss-glow);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.loading-text {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-secondary);
}
```

### Empty

```css
.empty-state {
  text-align: center;
  padding: var(--space-8);
}

.empty-state-icon {
  font-size: var(--text-lg);
  color: var(--text-muted);
  margin-bottom: var(--space-3);
}

.empty-state-message {
  font-family: var(--font-mono);
  font-size: var(--text-base);
  color: var(--text-secondary);
  margin-bottom: var(--space-2);
}

.empty-state-hint {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-muted);
}
```

### Error

```css
.error-message {
  padding: var(--space-3) var(--space-4);
  background: rgba(255, 51, 102, 0.1);
  border: 1px solid var(--danger);
  border-radius: 4px;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--danger);
}
```

### Success

```css
.success-message {
  padding: var(--space-3) var(--space-4);
  background: rgba(124, 184, 124, 0.1);
  border: 1px solid var(--moss-glow);
  border-radius: 4px;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--moss-glow);
}
```

---

## UI Terminology

Use clear, standard terms:

| Term | Usage |
|------|-------|
| Online / Offline | Connection status |
| Connecting | Attempting connection |
| Create | New item creation |
| Task | Work item |
| Loading... | Loading state |
| Profile | User identity |
| Submit | Form submission |
| Cancel | Dismiss action |
| Delete | Remove item |
| Share | Share with others |
| Copy | Copy to clipboard |

---

## Accessibility

### Requirements

- **Touch targets**: Minimum 44x44px
- **Color contrast**: WCAG AA (4.5:1 for text)
- **Focus states**: Visible cyan outline
- **Reduced motion**: Respect `prefers-reduced-motion`
- **Screen reader**: ARIA labels for status indicators

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

## Page Layout

```css
.page {
  min-height: 100vh;
  background: var(--void-black);
  padding: var(--space-6);
}

.page-header {
  margin-bottom: var(--space-6);
}

.page-title {
  font-family: var(--font-serif);
  font-size: var(--text-lg);
  color: var(--gold);
  letter-spacing: 0.05em;
}

.page-content {
  max-width: 800px;
}
```

---

## Quick Reference

**Colors:**
- Background: `#0a0a0a`
- Gold (headers): `#d4af37`
- Cyan (interactive): `#00d4aa`
- Moss (status): `#7cb87c`
- Danger: `#ff3366`
- Text: `#f5f5f5`

**Fonts:**
- Titles: `Cormorant Garamond`
- Body: `JetBrains Mono`

**Sizes (3 only):**
- Small: 14px
- Base: 16px
- Large: 24px

**Spacing (4pt grid):**
- 4, 8, 12, 16, 24, 32px

**Transitions (2 only):**
- Fast: 150ms
- Normal: 200ms

---

## Migration from v1

### What Changed

| v1 | v2 |
|----|-----|
| 7 type sizes | 3 type sizes |
| 3 transition speeds | 2 transition speeds |
| 5+ button variants | 3 button variants |
| Decorative animations | Functional animations only |
| Glowing hover states | Subtle background tints |
| Complex status orb | Simple status dot |
| Sacred terminology | Standard terminology |

### What Stayed

- Color palette (void, gold, cyan, moss, danger)
- Font pairing (Cormorant Garamond + JetBrains Mono)
- Dark terminal foundation
- Cyan focus rings
- Reduced motion support

---

*Design system v2 — Minimal terminal aesthetic*
*Created: 2026-01-19*
