# SyncEngine Design System

> **Aesthetic**: Cyber-Mystical Terminal
> **Principle**: Sacred geometry meets terminal elegance
> **Source**: syncengine-v0 reference implementation

---

## Design Philosophy

SyncEngine embodies a **cyber-mystical terminal aesthetic** — where sacred geometry meets the focused clarity of command-line interfaces. The design creates a sense of entering a sacred digital space, a "temple" for collective intention.

### Core Tenets

| Tenet | Expression |
|-------|------------|
| **Sacred Darkness** | Deep void backgrounds allow light elements to glow meaningfully |
| **Intentional Color** | Every color has semantic meaning (gold = sacred, cyan = technology, green = active) |
| **Terminal Clarity** | Monospace fonts and minimal chrome create focus |
| **Geometric Foundation** | Sacred geometry (Seed of Life) as ambient visual substrate |
| **Mystical Language** | UI copy uses spiritual terminology ("field", "manifest", "intentions") |

---

## Color System

### Primary Palette

```css
:root {
  /* === VOID (Backgrounds) === */
  --void-black: #0a0a0a;           /* Primary background */
  --void-lighter: #0a0e0f;         /* Elevated surfaces */
  --void-border: #1a1a1a;          /* Subtle borders */
  
  /* === MOSS GREEN (Nature, Growth, Status) === */
  --moss: #5a7a5a;                 /* Borders, inactive states */
  --moss-glow: #7cb87c;            /* Active status, success */
  --moss-bright: #39ff14;          /* Extreme emphasis (rare) */
  
  /* === CYAN (Technology, Links, Input) === */
  --cyan: #00d4aa;                 /* Links, tech terms, focus states */
  --cyan-glow: rgba(0, 212, 170, 0.3);
  
  /* === GOLD (Sacred, Important, Titles) === */
  --gold: #d4af37;                 /* Headings, sacred terms */
  --gold-glow: rgba(212, 175, 55, 0.3);
  
  /* === TEXT === */
  --text-primary: #f5f5f5;         /* Primary text */
  --text-secondary: rgba(245, 245, 245, 0.7);  /* Secondary text */
  --text-muted: rgba(245, 245, 245, 0.5);      /* Placeholder, hints */
  
  /* === SEMANTIC === */
  --danger: #ff3366;               /* Errors, destructive actions */
  --warning: #ff9f00;              /* Warnings */
  --info: #5f8fff;                 /* Information */
  --lilac: #c4a7d7;                /* Mystical accents */
}
```

### Color Semantics

| Color | Meaning | Usage |
|-------|---------|-------|
| **Gold** | Sacred, important | Page titles, section headers, "Flywheel of Gratitude" |
| **Cyan** | Technology, interaction | Links, focused inputs, "OrbitDB", "peer-to-peer" |
| **Moss Green** | Nature, status, growth | Status dots, borders, success states |
| **White** | Clarity, content | Body text, descriptions |
| **Lilac** | Mystical | Special spiritual elements |

---

## Typography

### Font Stack

```css
/* Headlines, sacred text — elegant serif */
--font-serif: 'Cormorant Garamond', Georgia, serif;

/* Body, technical, UI — monospace terminal */
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

/* Technical highlight — cyan */
.tech-term {
  color: var(--cyan);
}

/* Sacred highlight — gold, italic */
.sacred-term {
  color: var(--gold);
  font-style: italic;
}
```

---

## Sacred Geometry

### Seed of Life Background

The Seed of Life pattern appears as a subtle ambient element behind key UI areas.

```css
.seed-of-life-bg {
  position: absolute;
  inset: 0;
  background-image: url('/assets/seed-of-life.svg');
  background-repeat: no-repeat;
  background-position: center;
  background-size: 600px 600px;
  opacity: 0.15;
  pointer-events: none;
}
```

**SVG Pattern (seed-of-life.svg):**
```svg
<svg viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg">
  <g fill="none" stroke="#d4af37" stroke-width="0.5">
    <!-- Center circle -->
    <circle cx="100" cy="100" r="30"/>
    <!-- Six surrounding circles -->
    <circle cx="100" cy="70" r="30"/>
    <circle cx="126" cy="85" r="30"/>
    <circle cx="126" cy="115" r="30"/>
    <circle cx="100" cy="130" r="30"/>
    <circle cx="74" cy="115" r="30"/>
    <circle cx="74" cy="85" r="30"/>
  </g>
</svg>
```

---

## Components

### Status Indicator

The "field listening" / "field resonating" pattern:

```rust
#[component]
pub fn FieldStatus(status: FieldState) -> Element {
    let (label, is_active) = match status {
        FieldState::Listening => ("field listening", true),
        FieldState::Resonating => ("field resonating", true),
        FieldState::Dormant => ("field dormant", false),
    };
    
    rsx! {
        div { class: "field-status",
            span { 
                class: if is_active { "status-dot active" } else { "status-dot" }
            }
            span { class: "status-label", "{label}" }
        }
    }
}
```

```css
.field-status {
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

.status-dot.active {
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

```rust
#[component]
pub fn CategoryPills(
    categories: Vec<&str>,
    selected: String,
    on_select: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "category-pills",
            for cat in categories {
                button {
                    class: if selected == cat { "pill selected" } else { "pill" },
                    onclick: move |_| on_select.call(cat.to_string()),
                    "{cat}"
                }
            }
        }
    }
}
```

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

.input-hint {
  color: var(--text-muted);
  font-size: var(--text-xs);
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

/* Badge/tag button */
.btn-badge {
  padding: 0.375rem 0.75rem;
  background: transparent;
  border: 1px solid var(--moss);
  border-radius: 4px;
  color: var(--cyan);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  cursor: pointer;
}
```

### Intention Card (Expandable List Item)

```rust
#[component]
pub fn IntentionItem(
    title: String,
    has_matches: bool,
    expanded: Signal<bool>,
) -> Element {
    rsx! {
        div { class: "intention-item",
            button {
                class: "intention-header",
                onclick: move |_| expanded.toggle(),
                span { class: "expand-icon", 
                    if expanded() { "v" } else { ">" }
                }
                span { class: "intention-title", "{title}" }
            }
            if expanded() {
                div { class: "intention-content",
                    if !has_matches {
                        p { class: "no-matches",
                            "no matching intentions in the field"
                        }
                    }
                }
            }
        }
    }
}
```

```css
.intention-item {
  border-left: 2px solid var(--moss);
  margin-left: 1rem;
}

.intention-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem;
  background: transparent;
  border: none;
  color: var(--cyan);
  font-family: var(--font-mono);
  font-size: var(--text-base);
  cursor: pointer;
}

.expand-icon {
  color: var(--moss-glow);
}

.intention-content {
  padding: 0.5rem 1rem 1rem 1.5rem;
}

.no-matches {
  color: var(--text-muted);
  font-style: italic;
  font-size: var(--text-sm);
}
```

### Manifest Form

```rust
#[component]
pub fn ManifestForm(on_submit: EventHandler<Intention>) -> Element {
    let mut title = use_signal(String::new);
    let mut subtitle = use_signal(String::new);
    let mut category = use_signal(|| "general".to_string());
    let mut description = use_signal(String::new);
    
    rsx! {
        div { class: "manifest-form",
            header { class: "form-header",
                span { class: "plus-icon", "+" }
                span { "manifest new intention" }
                button { class: "close-btn", "×" }
            }
            
            div { class: "form-field",
                label { class: "input-label", "title" }
                input {
                    class: "input-field",
                    value: "{title}",
                    oninput: move |e| title.set(e.value())
                }
            }
            
            div { class: "form-field",
                label { class: "input-label", 
                    "subtitle "
                    span { class: "input-hint", "(optional)" }
                }
                input {
                    class: "input-field",
                    placeholder: "a brief tagline or context...",
                    value: "{subtitle}",
                    oninput: move |e| subtitle.set(e.value())
                }
            }
            
            div { class: "form-field",
                label { class: "input-label", "category" }
                CategoryPills {
                    categories: vec!["general", "intention", "offering", "collective"],
                    selected: category(),
                    on_select: move |c| category.set(c)
                }
            }
            
            div { class: "form-field",
                label { class: "input-label",
                    "description "
                    span { class: "input-hint", "(markdown supported)" }
                }
                textarea {
                    class: "input-field textarea",
                    placeholder: "add details, context, or notes...",
                    value: "{description}",
                    oninput: move |e| description.set(e.value())
                }
            }
        }
    }
}
```

```css
.manifest-form {
  background: var(--void-lighter);
  border: 1px solid var(--void-border);
  border-radius: 8px;
  padding: 1.5rem;
}

.form-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 1.5rem;
  font-family: var(--font-mono);
  color: var(--text-primary);
}

.plus-icon {
  color: var(--moss-glow);
}

.close-btn {
  margin-left: auto;
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: var(--text-lg);
  cursor: pointer;
}

.form-field {
  margin-bottom: 1.25rem;
}

.textarea {
  min-height: 100px;
  resize: vertical;
}
```

---

## Page Layouts

### Landing Page

```rust
#[component]
pub fn LandingPage() -> Element {
    rsx! {
        main { class: "landing",
            // Sacred geometry background
            div { class: "seed-of-life-bg" }
            
            header { class: "landing-header",
                h1 { class: "page-title", "Synchronicity Engine" }
                p { class: "tagline", "a decentralized organism of collective awakening" }
                
                FieldStatus { status: FieldState::Resonating }
                
                button { class: "btn-enter", "Enter the Field" }
            }
            
            section { class: "vision-section",
                h2 { class: "section-header", "The Vision" }
                p {
                    "The "
                    span { class: "sacred-term", "Synchronicity Engine" }
                    " is a "
                    span { class: "tech-term", "decentralized, peer-to-peer organism" }
                    " of collective awakening..."
                }
            }
        }
    }
}
```

```css
.landing {
  min-height: 100vh;
  background: var(--void-black);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 4rem 2rem;
  position: relative;
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

.vision-section {
  max-width: 700px;
  text-align: left;
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

.app-footer {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 1rem;
  text-align: center;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-muted);
}

.app-footer-message {
  /* "synchronicities are forming" */
  animation: fadeInOut 3s ease-in-out infinite;
}

@keyframes fadeInOut {
  0%, 100% { opacity: 0.5; }
  50% { opacity: 1; }
}
```

---

## UI Language Guide

Use this terminology consistently throughout the app:

| Standard Term | SyncEngine Term |
|---------------|-----------------|
| Connected / Online | Field resonating |
| Connecting | Field listening |
| Offline | Field dormant |
| Create new | Manifest new intention |
| Task / Item | Intention |
| Loading | Synchronicities are forming |
| User / Account | Steward |
| Server / Node | Temple |
| Dashboard / Home | Synchronicity Engine |
| Category | Category (keep) |
| Submit | Manifest |
| Cancel | Release |
| Delete | Dissolve |
| Share | Offer to the field |
| Shared list | Collective |
| Private list | Personal sanctuary |

---

## Animation Guidelines

### Principles

1. **Subtle over dramatic** — Animations should feel like natural breathing, not attention-grabbing
2. **Glow over movement** — Prefer opacity/shadow changes over positional movement
3. **Slow and meditative** — Use longer durations (300ms-1s) for a contemplative feel

### Standard Transitions

```css
/* Default transition */
--transition-fast: 150ms ease;
--transition-normal: 300ms ease;
--transition-slow: 500ms ease;
--transition-meditative: 1s cubic-bezier(0.4, 0, 0.2, 1);

/* Glow pulse for active elements */
@keyframes pulse {
  0%, 100% { opacity: 1; box-shadow: 0 0 10px currentColor; }
  50% { opacity: 0.7; box-shadow: 0 0 20px currentColor; }
}

/* Gentle fade for content */
@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

/* Status message breathing */
@keyframes breathe {
  0%, 100% { opacity: 0.5; }
  50% { opacity: 1; }
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
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

---

## File Structure

```
src/
├── main.rs
├── app.rs
├── theme/
│   ├── mod.rs
│   ├── colors.rs          # Color constants
│   ├── typography.rs      # Font definitions
│   └── tokens.rs          # CSS custom properties
├── components/
│   ├── mod.rs
│   ├── field_status.rs    # Status indicator
│   ├── category_pills.rs  # Category selector
│   ├── intention_item.rs  # Expandable list item
│   ├── manifest_form.rs   # Create intention form
│   ├── seed_of_life.rs    # Sacred geometry SVG
│   └── btn.rs             # Button variants
├── pages/
│   ├── landing.rs         # Landing page
│   ├── field.rs           # Main app view
│   └── profile.rs         # User profile
└── hooks/
    └── use_field_status.rs

assets/
├── style.css              # Global styles
├── seed-of-life.svg       # Sacred geometry pattern
└── fonts/                 # Self-hosted fonts (optional)
```

---

## Quick Reference

**Colors:**
- Background: `#0a0a0a`
- Gold (sacred): `#d4af37`
- Cyan (tech): `#00d4aa`  
- Moss (status): `#7cb87c`
- Text: `#f5f5f5`

**Fonts:**
- Titles: `Cormorant Garamond`
- Body: `JetBrains Mono`

**Key Language:**
- "field listening" / "field resonating"
- "manifest new intention"
- "synchronicities are forming"
- "Enter the Field"

---

*May this interface serve as a temple for collective intention.*
