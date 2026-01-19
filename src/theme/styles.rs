//! Global CSS styles for the Synchronicity Engine.
//!
//! Cyber-mystical terminal aesthetic from DESIGN_SYSTEM.md.

pub const GLOBAL_STYLES: &str = r#"
/* === CSS Custom Properties === */
:root {
  /* VOID (Backgrounds) */
  --void-black: #0a0a0a;
  --void-lighter: #0a0e0f;
  --void-border: #1a1a1a;

  /* MOSS GREEN (Nature, Growth, Status) */
  --moss: #5a7a5a;
  --moss-glow: #7cb87c;
  --moss-bright: #39ff14;

  /* CYAN (Technology, Links, Input) */
  --cyan: #00d4aa;
  --cyan-glow: rgba(0, 212, 170, 0.3);

  /* GOLD (Sacred, Important, Titles) */
  --gold: #d4af37;
  --gold-glow: rgba(212, 175, 55, 0.3);

  /* TEXT */
  --text-primary: #f5f5f5;
  --text-secondary: rgba(245, 245, 245, 0.7);
  --text-muted: rgba(245, 245, 245, 0.5);

  /* SEMANTIC */
  --danger: #ff3366;
  --warning: #ff9f00;
  --info: #5f8fff;
  --lilac: #c4a7d7;

  /* Typography */
  --font-serif: 'Cormorant Garamond', Georgia, serif;
  --font-mono: 'JetBrains Mono', 'SF Mono', 'Consolas', monospace;

  /* Type Scale */
  --text-xs: 0.75rem;
  --text-sm: 0.875rem;
  --text-base: 1rem;
  --text-lg: 1.125rem;
  --text-xl: 1.5rem;
  --text-2xl: 2rem;
  --text-3xl: 3rem;

  /* Transitions */
  --transition-fast: 150ms ease;
  --transition-normal: 300ms ease;
  --transition-slow: 500ms ease;
  --transition-meditative: 1s cubic-bezier(0.4, 0, 0.2, 1);
}

/* === Global Reset === */
*, *::before, *::after {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

html {
  font-size: 16px;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

body {
  font-family: var(--font-mono);
  background: var(--void-black);
  color: var(--text-primary);
  line-height: 1.7;
  min-height: 100vh;
}

/* === Typography === */
.page-title {
  font-family: var(--font-serif);
  font-size: var(--text-3xl);
  font-weight: 400;
  color: var(--gold);
  text-shadow: 0 0 30px var(--gold-glow);
  letter-spacing: 0.1em;
}

.section-header {
  font-family: var(--font-serif);
  font-size: var(--text-xl);
  font-weight: 400;
  font-style: italic;
  color: var(--gold);
}

.body-text {
  font-family: var(--font-mono);
  font-size: var(--text-base);
  font-weight: 400;
  color: var(--text-primary);
  line-height: 1.7;
}

.tech-term {
  color: var(--cyan);
}

.sacred-term {
  color: var(--gold);
  font-style: italic;
}

/* === Status Indicator === */
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

/* === Network Resonance Indicator === */
.network-resonance {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
}

.network-resonance.compact {
  gap: 0.5rem;
}

/* Resonance Orb Container */
.resonance-orb {
  position: relative;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.resonance-orb.compact {
  width: 20px;
  height: 20px;
}

/* Concentric Rings (visible when resonating) */
.resonance-ring {
  position: absolute;
  border-radius: 50%;
  border: 1px solid var(--moss);
  animation: ring-pulse 2s ease-in-out infinite;
}

.resonance-ring.outer {
  width: 100%;
  height: 100%;
  opacity: 0.4;
  animation-delay: 0s;
}

.resonance-ring.middle {
  width: 75%;
  height: 75%;
  opacity: 0.6;
  animation-delay: 0.5s;
}

.resonance-ring.outer.compact {
  width: 100%;
  height: 100%;
}

@keyframes ring-pulse {
  0%, 100% {
    transform: scale(1);
    opacity: 0.4;
  }
  50% {
    transform: scale(1.15);
    opacity: 0.7;
  }
}

/* Core Resonance Dot */
.resonance-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  position: relative;
  z-index: 1;
  transition: all var(--transition-normal);
}

/* State: Dormant (offline) */
.resonance-dot.dormant {
  background: var(--text-muted);
  box-shadow: none;
}

/* State: Seeking (connecting) */
.resonance-dot.seeking {
  background: var(--gold);
  box-shadow: 0 0 8px var(--gold-glow);
  animation: seeking-pulse 1.5s ease-in-out infinite;
}

@keyframes seeking-pulse {
  0%, 100% {
    box-shadow: 0 0 8px var(--gold-glow);
    transform: scale(1);
  }
  50% {
    box-shadow: 0 0 16px var(--gold-glow);
    transform: scale(1.1);
  }
}

/* State: Listening (connected, 0 peers) */
.resonance-dot.listening {
  background: var(--cyan);
  box-shadow: 0 0 10px var(--cyan-glow);
  animation: listening-glow 2s ease-in-out infinite;
}

@keyframes listening-glow {
  0%, 100% {
    box-shadow: 0 0 8px var(--cyan-glow);
  }
  50% {
    box-shadow: 0 0 14px var(--cyan-glow);
  }
}

/* State: Resonating (syncing with peers) */
.resonance-dot.resonating {
  background: var(--moss-glow);
  box-shadow: 0 0 12px var(--moss-glow);
  animation: resonating-pulse 1.8s ease-in-out infinite;
}

@keyframes resonating-pulse {
  0%, 100% {
    box-shadow: 0 0 10px var(--moss-glow);
    transform: scale(1);
  }
  50% {
    box-shadow: 0 0 20px var(--moss-glow), 0 0 30px rgba(124, 184, 124, 0.2);
    transform: scale(1.05);
  }
}

/* State: Dissonance (error) */
.resonance-dot.dissonance {
  background: var(--danger);
  box-shadow: 0 0 10px rgba(255, 51, 102, 0.5);
  animation: dissonance-flash 0.8s ease-in-out infinite;
}

@keyframes dissonance-flash {
  0%, 100% {
    opacity: 1;
    box-shadow: 0 0 10px rgba(255, 51, 102, 0.5);
  }
  50% {
    opacity: 0.7;
    box-shadow: 0 0 16px rgba(255, 51, 102, 0.7);
  }
}

/* Resonance Labels */
.resonance-label {
  color: var(--text-secondary);
  transition: color var(--transition-normal);
  white-space: nowrap;
}

.resonance-label.dormant {
  color: var(--text-muted);
}

.resonance-label.seeking {
  color: var(--gold);
  animation: seeking-text 1.5s ease-in-out infinite;
}

@keyframes seeking-text {
  0%, 100% { opacity: 0.7; }
  50% { opacity: 1; }
}

.resonance-label.listening {
  color: var(--cyan);
}

.resonance-label.resonating {
  color: var(--moss-glow);
}

.resonance-label.dissonance {
  color: var(--danger);
}

.resonance-label.compact {
  font-size: var(--text-xs);
}

/* Network Resonance Container (for dropdown positioning) */
.network-resonance-container {
  position: relative;
}

.network-resonance {
  cursor: pointer;
  transition: opacity 0.2s ease;
}

.network-resonance:hover {
  opacity: 0.8;
}

/* Dropdown Arrow */
.dropdown-arrow {
  margin-left: 0.5rem;
  font-size: var(--text-xs);
  color: var(--text-secondary);
  transition: transform 0.2s ease;
}

/* Network Debug Dropdown Panel */
.network-debug-dropdown {
  position: absolute;
  top: calc(100% + 0.5rem);
  right: 0;
  width: 320px;
  background: #121616;
  border: 1px solid var(--void-border);
  border-radius: 4px;
  padding: 1rem;
  z-index: 1000;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.8);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
}

.debug-title {
  font-family: var(--font-title);
  font-size: var(--text-base);
  color: var(--gold);
  margin: 0 0 0.75rem 0;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--void-border);
}

.debug-section {
  margin-bottom: 0.75rem;
}

.debug-section:last-child {
  margin-bottom: 0;
}

.debug-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.25rem 0;
}

.debug-row.full-id {
  flex-direction: column;
  align-items: flex-start;
}

.debug-label {
  color: var(--text-secondary);
  font-size: var(--text-xs);
}

.debug-value {
  color: var(--text-primary);
}

.debug-value.mono {
  font-family: var(--font-mono);
  color: var(--cyan);
}

.debug-value.error {
  color: var(--danger);
}

.debug-row.error {
  background: rgba(255, 100, 100, 0.1);
  padding: 0.25rem 0.5rem;
  border-radius: 2px;
  margin: 0.25rem 0;
}

.debug-copyable {
  width: 100%;
  margin-top: 0.25rem;
  padding: 0.5rem;
  background: var(--void-black);
  border-radius: 2px;
  overflow-x: auto;
}

.debug-code {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  color: var(--cyan);
  word-break: break-all;
}

.debug-no-info {
  color: var(--text-secondary);
  font-style: italic;
  margin: 0;
}

/* === Peer List === */
.peer-list {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  margin-top: 0.5rem;
  padding: 0.5rem;
  background: var(--void-black);
  border-radius: 2px;
}

.peer-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.25rem 0.5rem;
  border-radius: 2px;
  transition: background 0.2s ease;
}

.peer-row:hover {
  background: rgba(124, 184, 124, 0.05);
}

/* Peer Status Dot */
.peer-status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.peer-status-dot.online {
  background: var(--moss-glow);
  box-shadow: 0 0 6px var(--moss-glow);
  animation: peer-pulse 2s ease-in-out infinite;
}

.peer-status-dot.offline {
  background: var(--text-muted);
  opacity: 0.5;
}

@keyframes peer-pulse {
  0%, 100% {
    opacity: 1;
    box-shadow: 0 0 6px var(--moss-glow);
  }
  50% {
    opacity: 0.7;
    box-shadow: 0 0 10px var(--moss-glow);
  }
}

/* Peer ID */
.peer-id {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.peer-id.online {
  color: var(--cyan);
}

.peer-id.offline {
  color: var(--text-muted);
}

/* Peer Duration */
.peer-duration {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  color: var(--text-secondary);
  flex-shrink: 0;
}

/* === Buttons === */
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

.btn-manifest-new {
  font-family: var(--font-serif);
  font-style: italic;
  font-weight: 300;
  padding: 0.875rem 1.5rem;
  background: transparent;
  border: 1px solid rgba(124, 184, 124, 0.3);
  border-radius: 2px;
  color: var(--color-moss-glow);
  font-size: 1rem;
  letter-spacing: 0.05em;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  justify-content: center;
}

.btn-manifest-new:hover {
  border-color: var(--color-cyan);
  color: var(--color-cyan);
  box-shadow: 0 0 15px rgba(0, 212, 170, 0.2);
  transform: translateY(-1px);
}

.btn-manifest-new .btn-icon {
  color: var(--color-cyan);
  font-size: 1.25rem;
  font-style: normal;
  line-height: 1;
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

/* === Input Fields === */
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

/* === Category Pills === */
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

/* === Seed of Life Background === */
.seed-of-life-bg {
  position: absolute;
  inset: 0;
  background-image: url('assets/seed-of-life.svg');
  background-repeat: no-repeat;
  background-position: center;
  background-size: 600px 600px;
  opacity: 0.15;
  pointer-events: none;
}

/* === Page Layouts === */
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
  z-index: 1;
}

.tagline {
  font-family: var(--font-mono);
  font-size: var(--text-lg);
  color: var(--text-secondary);
  margin-top: 1rem;
}

.vision-section {
  max-width: 700px;
  text-align: left;
  z-index: 1;
}

/* === App Shell === */
.app-shell {
  min-height: 100vh;
  background: var(--void-black);
  padding: 2rem;
  position: relative;
}

/* Seed of Life background for Field view */
.app-shell::before {
  content: '';
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 800px;
  height: 800px;
  background-image: url('assets/seed-of-life.svg');
  background-repeat: no-repeat;
  background-position: center;
  background-size: contain;
  opacity: 0.03;
  pointer-events: none;
  z-index: 0;
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
  animation: breathe 3s ease-in-out infinite;
}

@keyframes breathe {
  0%, 100% { opacity: 0.5; }
  50% { opacity: 1; }
}

/* === Field Content Layout === */
.field-content {
  display: flex;
  gap: 2rem;
  min-height: calc(100vh - 200px);
  padding-bottom: 5rem; /* Ensure content doesn't overlap fixed footer */
}

/* === Realm Sidebar === */
.realm-sidebar {
  width: 280px;
  min-width: 280px;
  border-right: 1px solid var(--void-border);
  padding-right: 2rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.realm-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-top: 1rem;
  margin-bottom: 1rem;
}

.realm-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  background: transparent;
  border: 1px solid var(--void-border);
  border-radius: 4px;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: all 0.2s ease;
  text-align: left;
  width: 100%;
}

.realm-item:hover {
  border-color: var(--moss);
  background: var(--void-lighter);
}

.realm-item.selected {
  border-color: var(--moss-glow);
  background: var(--void-lighter);
  box-shadow: 0 0 10px rgba(124, 184, 124, 0.1);
}

.realm-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.realm-shared-badge {
  font-size: var(--text-xs);
  color: var(--cyan);
  padding: 0.125rem 0.375rem;
  border: 1px solid var(--cyan);
  border-radius: 2px;
  margin-left: 0.5rem;
}

.new-realm-input {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-top: 0.5rem;
}

.new-realm-actions {
  display: flex;
  gap: 0.5rem;
}

/* === Task Area === */
.task-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.manifest-input {
  display: flex;
  gap: 1rem;
  align-items: center;
}

.manifest-input .input-field {
  flex: 1;
}

/* === Intention List === */
.intention-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.intention-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  background: var(--void-lighter);
  border: 1px solid var(--void-border);
  border-radius: 4px;
  transition: all 0.2s ease;
}

.intention-item:hover {
  border-color: var(--moss);
}

.intention-toggle {
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.intention-toggle .check {
  font-size: var(--text-lg);
  color: var(--moss);
  transition: color 0.2s ease;
}

.intention-toggle .check.completed {
  color: var(--moss-glow);
}

.intention-toggle:hover .check {
  color: var(--moss-glow);
}

.intention-title {
  flex: 1;
  font-family: var(--font-mono);
  font-size: var(--text-base);
  color: var(--text-primary);
}

.intention-title.completed {
  text-decoration: line-through;
  color: var(--text-muted);
}

.intention-delete {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: var(--text-lg);
  padding: 0.25rem;
  opacity: 0;
  transition: all 0.2s ease;
}

.intention-item:hover .intention-delete {
  opacity: 1;
}

.intention-delete:hover {
  color: var(--danger);
}

/* === Empty & Loading States === */
.no-realm-selected {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  border: 1px dashed var(--void-border);
  border-radius: 8px;
  padding: 2rem;
  text-align: center;
}

.empty-state {
  color: var(--text-muted);
  font-style: italic;
  text-align: center;
  padding: 2rem;
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 300px;
}

.loading-message {
  font-family: var(--font-mono);
  font-size: var(--text-lg);
  color: var(--text-secondary);
  animation: breathe 2s ease-in-out infinite;
}

/* === Error Banner === */
.error-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  background: rgba(255, 51, 102, 0.1);
  border: 1px solid var(--danger);
  border-radius: 4px;
  margin-bottom: 1rem;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--danger);
}

.error-dismiss {
  background: transparent;
  border: 1px solid var(--danger);
  border-radius: 2px;
  padding: 0.25rem 0.5rem;
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  color: var(--danger);
  cursor: pointer;
  transition: all 0.2s ease;
}

.error-dismiss:hover {
  background: var(--danger);
  color: var(--void-black);
}

/* === Small Buttons === */
.btn-small {
  padding: 0.375rem 0.75rem;
  background: transparent;
  border: 1px solid var(--moss);
  border-radius: 4px;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-small:hover {
  border-color: var(--moss-glow);
  box-shadow: 0 0 10px rgba(124, 184, 124, 0.2);
}

.btn-small.btn-cancel {
  border-color: var(--text-muted);
  color: var(--text-muted);
}

.btn-small.btn-cancel:hover {
  border-color: var(--text-secondary);
  color: var(--text-secondary);
  box-shadow: none;
}

.btn-badge {
  padding: 0.375rem 0.75rem;
  background: transparent;
  border: 1px solid var(--moss);
  border-radius: 4px;
  color: var(--cyan);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-badge:hover {
  border-color: var(--moss-glow);
  box-shadow: 0 0 10px rgba(124, 184, 124, 0.2);
}

/* === Invite Panel === */
.invite-panel {
  background: #111111;
  border: 1px solid var(--void-border);
  border-radius: 8px;
  padding: 1.5rem;
  margin-top: 1rem;
}

.invite-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-family: var(--font-serif);
  font-size: var(--text-lg);
  font-weight: 400;
  margin-bottom: 1rem;
  color: var(--text-primary);
}

.panel-close-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: var(--text-xl);
  cursor: pointer;
  padding: 0.25rem 0.5rem;
  transition: color 0.2s ease;
  line-height: 1;
}

.panel-close-btn:hover {
  color: var(--text-primary);
}

.invite-generate-btn {
  width: 100%;
}

.invite-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
}

.loading-text {
  color: var(--text-secondary);
  font-style: italic;
  animation: breathe 2s ease-in-out infinite;
}

.invite-display {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
}

.invite-error {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0.75rem;
  background: rgba(255, 51, 102, 0.1);
  border: 1px solid var(--danger);
  border-radius: 4px;
  margin-bottom: 1rem;
  font-size: var(--text-sm);
  color: var(--danger);
}

/* === QR Code Display === */
.qr-code-container {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
  background: var(--void-black);
  border: 1px solid var(--void-border);
  border-radius: 4px;
}

.qr-code-image {
  display: block;
  max-width: 100%;
  height: auto;
}

.qr-code-fallback {
  padding: 1rem;
  text-align: center;
  background: var(--void-lighter);
  border: 1px solid var(--void-border);
  border-radius: 4px;
}

.qr-fallback-label {
  color: var(--text-muted);
  font-size: var(--text-sm);
  margin-bottom: 0.5rem;
}

.qr-fallback-text {
  font-size: var(--text-xs);
  color: var(--cyan);
  word-break: break-all;
  display: block;
  max-width: 200px;
}

/* === Invite Ticket Display === */
.invite-ticket-container {
  width: 100%;
  max-width: 280px;
}

.invite-ticket-text {
  background: var(--void-black);
  border: 1px solid var(--void-border);
  border-radius: 4px;
  padding: 0.75rem;
  overflow: hidden;
}

.invite-ticket-code {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--cyan);
  word-break: break-all;
  display: block;
  max-height: 120px;
  overflow-y: auto;
  user-select: all;
  cursor: text;
}

.invite-copy-btn {
  width: 100%;
  max-width: 280px;
  transition: all 0.3s ease;
}

.invite-copy-btn.copied {
  background: rgba(124, 184, 124, 0.2);
  border-color: var(--moss-glow);
  color: var(--moss-glow);
}

.invite-expiry {
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

.expiry-label {
  color: var(--text-muted);
}

.expiry-countdown {
  color: var(--warning);
  font-family: var(--font-mono);
}

.invite-realm-name {
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

.invite-new-btn {
  margin-top: 0.5rem;
}

/* === Modal Overlay === */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(10, 10, 10, 0.85);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 2rem;
}

.modal-content {
  background: #111111;
  border: 1px solid var(--void-border);
  border-radius: 8px;
  max-width: 480px;
  width: 100%;
  max-height: 90vh;
  overflow-y: auto;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.5rem;
  border-bottom: 1px solid var(--void-border);
}

.modal-close-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: var(--text-xl);
  cursor: pointer;
  padding: 0.25rem 0.5rem;
  transition: color 0.2s ease;
}

.modal-close-btn:hover {
  color: var(--text-primary);
}

.modal-body {
  padding: 1.5rem;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.75rem;
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--void-border);
}

/* === Join Realm Modal === */
.join-realm-modal {
  /* Additional styling specific to join modal */
}

.join-input {
  width: 100%;
  min-height: 100px;
  resize: vertical;
}

.join-error {
  padding: 0.75rem;
  background: rgba(255, 51, 102, 0.1);
  border: 1px solid var(--danger);
  border-radius: 4px;
  margin-bottom: 1rem;
  color: var(--danger);
  font-size: var(--text-sm);
}

.join-success {
  padding: 0.75rem;
  background: rgba(124, 184, 124, 0.1);
  border: 1px solid var(--moss-glow);
  border-radius: 4px;
  margin-bottom: 1rem;
  color: var(--moss-glow);
  font-size: var(--text-sm);
  text-align: center;
}

/* === Sacred Button === */
.btn-sacred {
  padding: 0.75rem 2rem;
  background: transparent;
  border: 1px solid var(--gold);
  border-radius: 4px;
  color: var(--gold);
  font-family: var(--font-mono);
  font-size: var(--text-base);
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn-sacred:hover {
  border-color: var(--gold);
  box-shadow: 0 0 20px var(--gold-glow);
  text-shadow: 0 0 10px var(--gold-glow);
}

/* === Ghost Button === */
.btn-ghost {
  padding: 0.75rem 2rem;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 4px;
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: var(--text-base);
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-ghost:hover {
  color: var(--text-primary);
  border-color: var(--void-border);
}

/* === Header Actions === */
.header-actions {
  display: flex;
  gap: 0.75rem;
  margin-left: auto;
  margin-right: 1rem;
}

.header-btn {
  padding: 0.5rem 1rem;
  background: transparent;
  border: 1px solid var(--moss);
  border-radius: 4px;
  color: var(--moss-glow);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: all 0.2s ease;
}

.header-btn:hover {
  border-color: var(--moss-glow);
  box-shadow: 0 0 10px rgba(124, 184, 124, 0.3);
}

.join-btn {
  border-color: var(--cyan);
  color: var(--cyan);
}

.join-btn:hover {
  border-color: var(--cyan);
  box-shadow: 0 0 10px var(--cyan-glow);
}

/* === Task Area Header === */
.task-area-header {
  display: flex;
  justify-content: flex-end;
  padding: 0.5rem 0;
  margin-bottom: 0.5rem;
  border-bottom: 1px solid var(--void-border);
}

.invite-toggle-btn {
  padding: 0.5rem 1rem;
  background: transparent;
  border: 1px solid var(--gold);
  border-radius: 4px;
  color: var(--gold);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: all 0.2s ease;
}

.invite-toggle-btn:hover {
  box-shadow: 0 0 10px var(--gold-glow);
}

.invite-toggle-btn.active {
  background: rgba(212, 175, 55, 0.1);
  box-shadow: 0 0 10px var(--gold-glow);
}

/* === Invite Sidebar === */
.invite-sidebar {
  width: 320px;
  min-width: 280px;
  background: var(--void-lighter);
  border-left: 1px solid var(--void-border);
  padding: 1rem;
  overflow-y: auto;
}

/* === Inline Link Button === */
.inline-link-btn {
  background: none;
  border: none;
  color: var(--cyan);
  font-family: var(--font-mono);
  font-size: inherit;
  cursor: pointer;
  text-decoration: underline;
  padding: 0;
}

.inline-link-btn:hover {
  color: var(--cyan);
  text-shadow: 0 0 10px var(--cyan-glow);
}

/* === Hint Text === */
.hint-text {
  color: var(--text-muted);
  font-size: var(--text-sm);
  margin-top: 1rem;
}

/* === Accessibility === */
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

/* === Responsive Layout for Narrow Windows === */
@media (max-width: 900px) {
  .app-shell {
    padding: 1rem;
  }

  .realm-sidebar {
    width: 180px;
    min-width: 160px;
    padding-right: 1rem;
  }

  .invite-sidebar {
    width: 260px;
    min-width: 220px;
    padding: 0.75rem;
  }

  .invite-panel {
    padding: 1rem;
  }

  .qr-code-container {
    padding: 0.5rem;
  }

  .qr-code-image {
    max-width: 150px;
    max-height: 150px;
  }

  .invite-ticket-container {
    max-width: 220px;
  }

  .invite-copy-btn {
    max-width: 220px;
    padding: 0.5rem 1rem;
    font-size: var(--text-sm);
  }

  .modal-content {
    max-width: 90%;
    margin: 1rem;
  }
}

@media (max-width: 700px) {
  .field-content {
    flex-direction: column;
    gap: 1rem;
  }

  .realm-sidebar {
    width: 100%;
    min-width: unset;
    border-right: none;
    border-bottom: 1px solid var(--void-border);
    padding-right: 0;
    padding-bottom: 1rem;
  }

  .invite-sidebar {
    width: 100%;
    min-width: unset;
    border-left: none;
    border-top: 1px solid var(--void-border);
    padding-top: 1rem;
  }

  .app-header {
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .header-actions {
    margin-left: 0;
    margin-right: 0;
  }
}

/* === Unified Field View Layout === */
.field-content-unified {
  display: flex;
  gap: 2rem;
  min-height: calc(100vh - 200px);
  padding-bottom: 5rem;
}

.unified-main {
  flex: 1;
}

/* === Unified Field View === */
.unified-field-view {
  width: 100%;
  padding: 0 1rem 5rem 1rem;
}

.realm-sections {
  display: flex;
  flex-direction: column;
  gap: 2rem;
}

/* === Realm Section === */
.realm-section {
  border-left: 2px solid var(--moss);
  padding-left: 1.5rem;
  transition: border-color 0.3s ease;
}

.realm-section:hover {
  border-color: var(--moss-glow);
}

/* Realm Header (collapsible) */
.realm-header {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.5rem 0;
  background: transparent;
  border: none;
  cursor: pointer;
  text-align: left;
  transition: all 0.2s ease;
}

.realm-header:hover .realm-title {
  text-shadow: 0 0 20px var(--gold-glow);
}

.expand-icon {
  color: var(--moss-glow);
  font-size: var(--text-sm);
  transition: transform 0.2s ease;
  width: 1rem;
  text-align: center;
}

.realm-title {
  flex: 1;
  margin: 0;
  transition: text-shadow 0.3s ease;
}

.realm-meta {
  display: flex;
  gap: 0.5rem;
  align-items: center;
  margin-left: auto;
}

.realm-badge {
  padding: 0.25rem 0.5rem;
  border-radius: 3px;
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  border: 1px solid;
}

.shared-badge {
  border-color: var(--cyan);
  color: var(--cyan);
  background: rgba(0, 212, 170, 0.1);
}

.count-badge {
  border-color: var(--moss);
  color: var(--text-secondary);
  background: transparent;
}

/* Realm Invite Button */
.realm-invite-btn {
  padding: 0.25rem 0.5rem;
  border-radius: 3px;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  border: 1px solid var(--gold);
  color: var(--gold);
  background: transparent;
  cursor: pointer;
  transition: all 0.2s ease;
  margin-left: 0.5rem;
}

.realm-invite-btn:hover {
  background: rgba(212, 175, 55, 0.1);
  box-shadow: 0 0 8px var(--gold-glow);
  border-color: var(--gold);
}

/* Realm Content (expandable) */
.realm-content {
  margin-top: 1rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Realm Manifest Input */
.realm-manifest-input {
  display: flex;
  gap: 1rem;
  align-items: center;
  padding: 0.5rem;
  background: rgba(10, 14, 15, 0.3);
  border-radius: 4px;
  border: 1px solid var(--void-border);
}

.realm-manifest-input .input-field {
  flex: 1;
}

/* Realm Task List */
.realm-task-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-left: 0.5rem;
}

/* === Quest Card Grid === */
.realm-quest-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(min(300px, 100%), 1fr));
  gap: 1.5rem;
  padding: 1rem 0;
  margin-left: 0.5rem;
}

/* Responsive breakpoints for quest cards */
/* Wide screens: Show 3 columns */
@media (min-width: 1400px) {
  .realm-quest-grid {
    grid-template-columns: repeat(3, 1fr);
    gap: 2rem;
  }
}

/* Large screens: Show 2 columns */
@media (min-width: 1025px) and (max-width: 1399px) {
  .realm-quest-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 1.5rem;
  }
}

@media (max-width: 1024px) {
  .realm-quest-grid {
    grid-template-columns: repeat(auto-fill, minmax(min(280px, 100%), 1fr));
    gap: 1rem;
  }
}

@media (max-width: 768px) {
  .realm-quest-grid {
    grid-template-columns: 1fr;
    gap: 1rem;
    margin-left: 0;
    padding: 0.5rem 0;
  }
}

@media (max-width: 500px) {
  .realm-quest-grid {
    gap: 0.75rem;
  }
}

.quest-card-wrapper {
  position: relative;
  transition: transform 0.2s ease-in-out;
}

.quest-card-wrapper:hover {
  transform: translateY(-2px);
}

.quest-card-delete {
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
  width: 2rem;
  height: 2rem;
  border-radius: 50%;
  background: rgba(10, 10, 10, 0.7);
  border: 1px solid var(--gold);
  color: var(--gold);
  font-size: 1.5rem;
  line-height: 1;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.2s ease-in-out, background 0.2s ease-in-out;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10;
}

.quest-card-wrapper:hover .quest-card-delete {
  opacity: 1;
}

.quest-card-delete:hover {
  background: rgba(212, 175, 55, 0.2);
  border-color: var(--gold-bright);
  color: var(--gold-bright);
}

.quest-card-delete:active {
  transform: scale(0.95);
}

.empty-task-state {
  color: var(--text-muted);
  font-style: italic;
  font-size: var(--text-sm);
  padding: 1rem;
  text-align: center;
  border: 1px dashed var(--void-border);
  border-radius: 4px;
  grid-column: 1 / -1; /* Span full width in grid */
}

/* === Create Realm Section === */
.create-realm-section {
  margin-top: 3rem;
  padding-top: 2rem;
  border-top: 1px solid var(--void-border);
  display: flex;
  justify-content: center;
}

.create-realm-btn {
  padding: 0.75rem 1.5rem;
  font-size: var(--text-sm);
}

.new-realm-form {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  max-width: 400px;
  width: 100%;
  padding: 1rem;
  background: var(--void-lighter);
  border: 1px solid var(--void-border);
  border-radius: 4px;
}

.form-actions {
  display: flex;
  gap: 0.5rem;
  justify-content: flex-end;
}

/* Empty Realms State */
.empty-realms-state {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 200px;
  border: 1px dashed var(--void-border);
  border-radius: 8px;
  padding: 2rem;
  text-align: center;
}

/* === Responsive Adjustments for Unified View === */
@media (max-width: 700px) {
  .unified-field-view {
    padding: 0 0.5rem 5rem 0.5rem;
  }

  .realm-section {
    padding-left: 1rem;
  }

  .realm-manifest-input {
    flex-direction: column;
    align-items: stretch;
  }

  .realm-manifest-input .btn-primary {
    width: 100%;
  }
}

/* === Profile Page === */
.profile-page {
  min-height: 100vh;
  padding: 0;
  position: relative;
  display: flex;
  flex-direction: column;
}

.profile-header {
  padding: 2rem 2rem 1rem 2rem;
  width: 100%;
}

.back-button {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1.5rem;
  background: transparent;
  border: 1px solid rgba(212, 175, 55, 0.2);
  border-radius: 4px;
  color: var(--gold);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: all 150ms ease;
  text-decoration: none;
}

.back-button:hover {
  background: rgba(212, 175, 55, 0.1);
  border-color: var(--gold);
  box-shadow: 0 0 15px rgba(212, 175, 55, 0.2);
  transform: translateX(-2px);
}

.profile-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 2rem;
  width: 100%;
  flex: 1;
}

.profile-left, .profile-right {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

/* Presence Card */
.presence-card {
  background: rgba(15, 15, 15, 0.6);
  border: 1px solid rgba(212, 175, 55, 0.2);
  border-radius: 8px;
  padding: 2rem;
  backdrop-filter: blur(12px);
  transition: all 300ms ease;
}

.presence-card:hover {
  border-color: rgba(212, 175, 55, 0.4);
  box-shadow: 0 0 20px rgba(212, 175, 55, 0.1);
}

.section-title {
  font-family: var(--font-serif);
  font-size: var(--text-xl);
  color: var(--gold);
  margin-bottom: 1.5rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
}

/* QR Signature */
.qr-signature {
  display: block;
  width: 100%;
}

.qr-signature svg {
  width: 100%;
  height: auto;
  display: block;
}

.qr-label {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

/* Identity Fields */
.node-signature, .endpoint-address {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem;
  background: rgba(0, 0, 0, 0.3);
  border-radius: 4px;
  margin: 0.5rem 0;
}

.node-signature .label,
.endpoint-address .label {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-muted);
  text-transform: uppercase;
}

.node-signature .value,
.endpoint-address .value {
  font-family: var(--font-mono);
  color: var(--cyan);
  font-size: var(--text-sm);
}

.copy-button {
  padding: 0.25rem 0.75rem;
  background: transparent;
  border: 1px solid var(--cyan);
  color: var(--cyan);
  border-radius: 4px;
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  cursor: pointer;
  transition: all 150ms ease;
}

.copy-button:hover {
  background: var(--cyan);
  color: var(--void-black);
}

.copy-button.copied {
  border-color: var(--moss-glow);
  color: var(--moss-glow);
  background: rgba(124, 184, 124, 0.1);
}

.connected-since {
  margin: 1rem 0;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--moss-glow);
  text-align: center;
}

/* Peer List */
.peer-list {
  background: rgba(15, 15, 15, 0.6);
  border: 1px solid rgba(212, 175, 55, 0.2);
  border-radius: 8px;
  padding: 2rem;
  backdrop-filter: blur(12px);
}

.peer-section {
  margin: 1.5rem 0;
}

.peer-section-header {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-muted);
  margin-bottom: 1rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

/* Peer Card */
.peer-card {
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(245, 245, 245, 0.1);
  border-radius: 6px;
  padding: 1rem;
  margin: 0.5rem 0;
  transition: all 150ms ease;
}

.peer-card:hover {
  transform: translateY(-2px);
  border-color: rgba(212, 175, 55, 0.3);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.peer-status {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 0.5rem;
}

.peer-status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.peer-status-dot.online {
  background: var(--moss-glow);
  box-shadow: 0 0 8px var(--moss-glow);
  animation: pulse 2s ease-in-out infinite;
}

.peer-status-dot.offline {
  background: var(--text-muted);
}

.peer-name {
  font-weight: 600;
  color: var(--gold);
  font-family: var(--font-mono);
}

.peer-id {
  font-family: var(--font-mono);
  color: var(--cyan);
  font-size: var(--text-sm);
}

.peer-last-seen {
  margin-left: auto;
  font-size: var(--text-xs);
  color: var(--text-muted);
}

.peer-metrics {
  font-size: var(--text-xs);
  color: var(--text-muted);
  display: flex;
  gap: 1rem;
  margin: 0.25rem 0 0.75rem 1.5rem;
}

.peer-actions {
  display: flex;
  gap: 0.5rem;
  margin-left: 1.5rem;
}

.empty-state {
  text-align: center;
  padding: 3rem 1rem;
  color: var(--text-muted);
}

.empty-state .hint {
  font-size: var(--text-sm);
  margin-top: 0.5rem;
}

/* Stewardship Stats */
.stewardship-stats {
  background: rgba(15, 15, 15, 0.6);
  border: 1px solid rgba(212, 175, 55, 0.2);
  border-radius: 8px;
  padding: 2rem;
  backdrop-filter: blur(12px);
}

.subtitle {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-muted);
  margin-bottom: 1.5rem;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1rem;
  margin: 1.5rem 0;
}

.stat-box {
  text-align: center;
  padding: 1.5rem 1rem;
  background: rgba(0, 0, 0, 0.3);
  border-radius: 4px;
  border: 1px solid rgba(245, 245, 245, 0.05);
}

.stat-value {
  font-size: var(--text-3xl);
  font-weight: 600;
  color: var(--gold);
  margin-bottom: 0.5rem;
  font-family: var(--font-mono);
}

.stat-label {
  font-size: var(--text-xs);
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-family: var(--font-mono);
}

/* Profile Nav Button */
.profile-nav-button {
  font-size: 1.25rem;
  padding: 0.5rem;
  color: var(--gold);
  background: transparent;
  border: 1px solid rgba(212, 175, 55, 0.2);
  border-radius: 50%;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 150ms ease;
}

.profile-nav-button:hover {
  border-color: var(--gold);
  box-shadow: 0 0 20px rgba(212, 175, 55, 0.2);
  transform: scale(1.05);
}

.profile-nav-button.active {
  background: rgba(212, 175, 55, 0.1);
  border-color: var(--gold);
  box-shadow: 0 0 20px var(--gold-glow);
}

/* Profile Page Responsive */
@media (max-width: 900px) {
  .profile-content {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 700px) {
  .profile-page {
    padding: 1rem;
  }

  .stats-grid {
    grid-template-columns: 1fr;
  }

  .peer-status {
    flex-wrap: wrap;
  }

  .peer-last-seen {
    flex-basis: 100%;
    margin-left: 1.5rem;
  }
}

/* ═══════════════════════════════════════════════════════════════════════
   GOLDEN RECTANGLE CARD SYSTEM
   ═══════════════════════════════════════════════════════════════════════ */

/* === Golden Ratio Variables === */
:root {
  --phi: 1.618;
  --phi-inv: 0.618;
  --card-base-width: 500px;

  --card-portrait-width: var(--card-base-width);
  --card-portrait-height: calc(var(--card-base-width) * var(--phi));

  --card-landscape-width: calc(var(--card-base-width) * var(--phi));
  --card-landscape-height: var(--card-base-width);

  --split-major: 61.8%;
  --split-minor: 38.2%;

  /* Fibonacci spacing scale */
  --spacing-phi-xs: 5px;
  --spacing-phi-sm: 8px;
  --spacing-phi-md: 13px;
  --spacing-phi-lg: 21px;
  --spacing-phi-xl: 34px;
  --spacing-phi-2xl: 55px;
}

/* === Golden Card Base === */
.golden-card {
  position: relative;
  background: linear-gradient(135deg, #0f0f0f 0%, #0a0a0a 100%);
  border-radius: 8px;
  overflow: hidden;
  width: 100%;
  max-width: 1100px;
  flex-shrink: 1;

  box-shadow:
    0 4px 12px rgba(0, 0, 0, 0.6),
    0 12px 40px rgba(0, 0, 0, 0.4),
    0 0 0 1px rgba(212, 175, 55, 0.1);

  transition: transform 0.4s cubic-bezier(0.34, 1.56, 0.64, 1),
              box-shadow 0.4s ease;
}

.golden-card--portrait {
  aspect-ratio: 1 / 1.618;
}

.golden-card--landscape {
  aspect-ratio: 1.618 / 1;
}

.golden-card.interactive {
  cursor: pointer;
}

.golden-card.interactive:hover {
  transform: translateY(-4px) scale(1.01);
  box-shadow:
    0 8px 24px rgba(0, 0, 0, 0.7),
    0 16px 56px rgba(0, 0, 0, 0.5),
    0 0 0 1px rgba(212, 175, 55, 0.3);
}

/* Sacred geometry background (Seed of Life pattern) */
.golden-card::before {
  content: '';
  position: absolute;
  inset: 0;
  z-index: 0;
  opacity: 0.03;
  background-image: radial-gradient(circle at 50% 50%, rgba(212, 175, 55, 0.1) 1px, transparent 1px);
  background-size: 20px 20px;
  transition: opacity 0.6s ease;
}

.golden-card.interactive:hover::before {
  opacity: 0.08;
  animation: pulse-sacred 3s ease-in-out infinite;
}

@keyframes pulse-sacred {
  0%, 100% { opacity: 0.08; }
  50% { opacity: 0.12; }
}

/* === Card Interior Layout === */
.golden-card__interior {
  display: grid;
  height: 100%;
  width: 100%;
  position: relative;
  z-index: 1;
}

.golden-card--portrait .golden-card__interior {
  grid-template-rows: var(--split-minor) var(--split-major);
}

.golden-card--landscape .golden-card__interior {
  grid-template-columns: var(--split-minor) var(--split-major);
}

/* === Image Area === */
.card-image-area {
  position: relative;
  background: linear-gradient(135deg, #1a1a1a, #0f0f0f);
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
}

.card-image__img,
.card-image__avatar,
.card-image__quest {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.6s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.golden-card.interactive:hover .card-image__img,
.golden-card.interactive:hover .card-image__avatar,
.golden-card.interactive:hover .card-image__quest {
  transform: scale(1.05);
}

.card-image__default {
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgba(212, 175, 55, 0.3);
  font-size: 48px;
}

.default-avatar-icon,
.default-quest-icon {
  width: 60%;
  height: 60%;
  opacity: 0.3;
}

/* Image overlays */
.card-image__overlay {
  position: absolute;
  bottom: 1rem;
  left: 50%;
  transform: translateX(-50%);
}

.card-image__upload-icon {
  position: absolute;
  bottom: 8px;
  right: 8px;
}

.card-image__badge {
  position: absolute;
  top: 1rem;
  right: 1rem;
  padding: 0.5rem 1rem;
  background: rgba(124, 184, 124, 0.9);
  color: #0a0a0a;
  font-family: var(--font-mono);
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  border-radius: 4px;
}

.card-image__loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 0.875rem;
}

.card-image__error {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--danger);
  font-family: var(--font-mono);
  font-size: 0.875rem;
  padding: 1rem;
  text-align: center;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid rgba(0, 212, 170, 0.2);
  border-top-color: var(--cyan);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* === Content Area === */
.card-content {
  display: flex;
  flex-direction: column;
  padding: var(--spacing-phi-lg);
  gap: 0.5rem;
  overflow: hidden;
  min-height: 0;
}

/* === Card Header === */
.card-header {
  border-bottom: 1px solid rgba(212, 175, 55, 0.2);
  padding-bottom: 0.375rem;
  margin-bottom: 0.5rem;
}

.card-header__title {
  font-family: var(--font-serif);
  font-size: 1rem;
  font-weight: 600;
  color: var(--gold);
  margin: 0 0 0.25rem 0;
  letter-spacing: 0.02em;
  line-height: 1.3;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-header__subtitle {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: rgba(124, 184, 124, 0.8);
  margin-bottom: 0.25rem;
}

.card-header__link {
  font-family: var(--font-mono);
  font-size: 0.875rem;
  color: var(--cyan);
  text-decoration: none;
  transition: color 0.2s ease;
  display: inline-block;
  margin-top: 0.25rem;
}

.card-header__link:hover {
  color: rgba(0, 212, 170, 0.7);
  text-decoration: underline;
}

/* Editable header */
.card-header--editing {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-phi-sm);
}

.editable-input {
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(124, 184, 124, 0.3);
  border-radius: 4px;
  padding: 0.5rem;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 0.875rem;
  transition: border-color 0.2s ease;
}

.editable-input:focus {
  outline: none;
  border-color: var(--cyan);
  box-shadow: 0 0 0 2px rgba(0, 212, 170, 0.1);
}

.editable-title {
  font-size: 1.25rem;
  font-weight: 600;
}

/* === Gallery === */
.card-gallery-section {
  margin: var(--spacing-phi-sm) 0;
}

.card-gallery__title {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--text-muted);
  margin-bottom: var(--spacing-phi-xs);
}

.card-gallery {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(48px, 1fr));
  gap: 8px;
}

.card-gallery__item {
  position: relative;
  aspect-ratio: 1 / 1;
  border-radius: 4px;
  overflow: hidden;
  border: 1px solid rgba(212, 175, 55, 0.2);
  cursor: pointer;
  transition: transform 0.2s ease, border-color 0.2s ease;
}

.card-gallery__item:hover {
  transform: scale(1.1);
  border-color: rgba(0, 212, 170, 0.6);
}

.card-gallery__img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.card-gallery__placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(26, 26, 26, 0.5);
  color: rgba(212, 175, 55, 0.3);
  font-family: var(--font-mono);
  font-size: 1.5rem;
}

.card-gallery__label {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: linear-gradient(to top, rgba(0, 0, 0, 0.9), transparent);
  padding: 0.25rem;
  font-family: var(--font-mono);
  font-size: 0.625rem;
  color: var(--text-primary);
  text-align: center;
  text-overflow: ellipsis;
  white-space: nowrap;
  overflow: hidden;
}

/* === Markdown Content === */
.card-markdown-section {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.card-markdown {
  flex: 1;
  overflow-y: auto;
  font-family: var(--font-mono);
  font-size: 0.875rem;
  line-height: 1.618;
  color: rgba(255, 255, 255, 0.85);

  scrollbar-width: thin;
  scrollbar-color: rgba(212, 175, 55, 0.3) transparent;
}

.card-markdown::-webkit-scrollbar {
  width: 6px;
}

.card-markdown::-webkit-scrollbar-track {
  background: transparent;
}

.card-markdown::-webkit-scrollbar-thumb {
  background: rgba(212, 175, 55, 0.3);
  border-radius: 3px;
}

.card-markdown--collapsed {
  max-height: 100px;
  overflow: hidden;
  position: relative;
}

.card-markdown--collapsed::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 40px;
  background: linear-gradient(to bottom, transparent, #0a0a0a);
  pointer-events: none;
}

.card-markdown h1, .card-markdown h2, .card-markdown h3 {
  font-family: var(--font-serif);
  color: var(--gold);
  margin: 1.2em 0 0.4em 0;
  line-height: 1.3;
}

.card-markdown h1 { font-size: 1.5rem; }
.card-markdown h2 { font-size: 1.25rem; }
.card-markdown h3 { font-size: 1.1rem; }

.card-markdown p {
  margin-bottom: 0.8em;
}

.card-markdown strong {
  color: rgba(212, 175, 55, 0.9);
  font-weight: 600;
}

.card-markdown em {
  color: rgba(0, 212, 170, 0.9);
  font-style: italic;
}

.card-markdown a {
  color: var(--cyan);
  border-bottom: 1px solid rgba(0, 212, 170, 0.3);
  text-decoration: none;
  transition: border-color 0.2s ease;
}

.card-markdown a:hover {
  border-bottom-color: var(--cyan);
}

.card-markdown code {
  background: rgba(212, 175, 55, 0.1);
  border: 1px solid rgba(212, 175, 55, 0.2);
  border-radius: 3px;
  padding: 2px 6px;
  color: rgba(212, 175, 55, 0.95);
  font-size: 0.85em;
}

.card-markdown pre {
  background: rgba(0, 0, 0, 0.5);
  border: 1px solid rgba(124, 184, 124, 0.2);
  border-radius: 4px;
  padding: 1rem;
  overflow-x: auto;
  margin: 1em 0;
}

.card-markdown pre code {
  background: none;
  border: none;
  padding: 0;
}

.card-markdown ul, .card-markdown ol {
  margin-left: 1.5em;
  margin-bottom: 0.8em;
}

.card-markdown li {
  margin-bottom: 0.3em;
}

.card-markdown blockquote {
  border-left: 3px solid var(--gold);
  padding-left: 1rem;
  margin: 1em 0;
  color: var(--text-secondary);
  font-style: italic;
}

/* === Markdown Editor === */
.markdown-editor {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-phi-sm);
  flex: 1;
  min-height: 0;
}

.markdown-toolbar {
  display: flex;
  gap: 4px;
  border-bottom: 1px solid rgba(212, 175, 55, 0.2);
  padding-bottom: 4px;
}

.markdown-toolbar button {
  padding: 4px 12px;
  background: transparent;
  border: 1px solid rgba(212, 175, 55, 0.2);
  border-radius: 4px;
  color: rgba(212, 175, 55, 0.8);
  font-family: var(--font-mono);
  font-size: 0.75rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.markdown-toolbar button:hover {
  background: rgba(212, 175, 55, 0.1);
  border-color: rgba(212, 175, 55, 0.4);
}

.markdown-toolbar button.active {
  background: rgba(212, 175, 55, 0.2);
  border-color: rgba(212, 175, 55, 0.6);
  color: var(--gold);
}

.markdown-textarea {
  width: 100%;
  min-height: 200px;
  flex: 1;
  padding: var(--spacing-phi-sm);
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(124, 184, 124, 0.3);
  border-radius: 4px;
  color: rgba(255, 255, 255, 0.85);
  font-family: var(--font-mono);
  font-size: 0.875rem;
  line-height: 1.618;
  resize: vertical;
  transition: border-color 0.2s ease;
}

.markdown-textarea:focus {
  outline: none;
  border-color: var(--cyan);
  box-shadow: 0 0 0 2px rgba(0, 212, 170, 0.1);
}

.markdown-preview {
  flex: 1;
  min-height: 200px;
  padding: var(--spacing-phi-sm);
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid rgba(212, 175, 55, 0.2);
  border-radius: 4px;
  overflow-y: auto;
}

.markdown-renderer {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-phi-sm);
  flex: 1;
  min-height: 0;
}

.markdown-toggle {
  align-self: flex-start;
  padding: 0.375rem 0.75rem;
  background: transparent;
  border: 1px solid rgba(0, 212, 170, 0.3);
  border-radius: 4px;
  color: var(--cyan);
  font-family: var(--font-mono);
  font-size: 0.75rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.markdown-toggle:hover {
  background: rgba(0, 212, 170, 0.1);
  border-color: var(--cyan);
}

.expand-toggle {
  margin-top: var(--spacing-phi-sm);
  padding: 0.5rem 1rem;
  background: transparent;
  border: 1px solid rgba(0, 212, 170, 0.3);
  border-radius: 4px;
  color: var(--cyan);
  font-family: var(--font-mono);
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.expand-toggle:hover {
  background: rgba(0, 212, 170, 0.1);
  border-color: var(--cyan);
}

/* === Image Upload === */
.image-upload {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-phi-sm);
}

.image-upload-btn {
  padding: 0.75rem 1.5rem;
  background: linear-gradient(135deg, rgba(0, 212, 170, 0.2), rgba(0, 212, 170, 0.1));
  border: 1px solid rgba(0, 212, 170, 0.4);
  border-radius: 6px;
  color: var(--cyan);
  font-family: var(--font-mono);
  font-size: 0.875rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
}

.image-upload-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, rgba(0, 212, 170, 0.3), rgba(0, 212, 170, 0.15));
  border-color: var(--cyan);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 212, 170, 0.2);
}

.image-upload-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.image-upload-btn--icon {
  width: 40px;
  height: 40px;
  padding: 0;
  background: rgba(10, 10, 10, 0.85);
  border: 1px solid rgba(124, 184, 124, 0.4);
  border-radius: 50%;
  color: var(--moss-glow);
  font-size: 1.25rem;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.3s ease;
  backdrop-filter: blur(4px);
}

.image-upload-btn--icon:hover:not(:disabled) {
  background: rgba(10, 10, 10, 0.95);
  border-color: var(--moss-glow);
  transform: scale(1.1);
  box-shadow: 0 4px 12px rgba(124, 184, 124, 0.3);
}

.image-upload-btn--icon:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.image-upload__error {
  padding: 0.5rem;
  background: rgba(255, 51, 102, 0.1);
  border: 1px solid rgba(255, 51, 102, 0.3);
  border-radius: 4px;
  color: var(--danger);
  font-family: var(--font-mono);
  font-size: 0.75rem;
}

/* === Card Actions === */
.card-actions {
  display: flex;
  gap: var(--spacing-phi-sm);
  margin-top: var(--spacing-phi-sm);
  padding-top: var(--spacing-phi-sm);
  border-top: 1px solid rgba(212, 175, 55, 0.2);
}

.btn-primary {
  flex: 1;
  padding: 0.75rem 1.5rem;
  background: linear-gradient(135deg, rgba(212, 175, 55, 0.3), rgba(212, 175, 55, 0.15));
  border: 1px solid rgba(212, 175, 55, 0.5);
  border-radius: 6px;
  color: var(--gold);
  font-family: var(--font-mono);
  font-size: 0.875rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn-primary:hover {
  background: linear-gradient(135deg, rgba(212, 175, 55, 0.4), rgba(212, 175, 55, 0.2));
  border-color: var(--gold);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(212, 175, 55, 0.3);
}

.btn-secondary {
  flex: 1;
  padding: 0.75rem 1.5rem;
  background: transparent;
  border: 1px solid rgba(124, 184, 124, 0.3);
  border-radius: 6px;
  color: var(--moss-glow);
  font-family: var(--font-mono);
  font-size: 0.875rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn-secondary:hover {
  background: rgba(124, 184, 124, 0.1);
  border-color: var(--moss-glow);
}

/* === Card Footer === */
.card-footer {
  display: flex;
  gap: var(--spacing-phi-md);
  flex-wrap: wrap;
  padding-top: var(--spacing-phi-sm);
  border-top: 1px solid rgba(212, 175, 55, 0.1);
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--text-muted);
}

.card-category {
  padding: 0.25rem 0.75rem;
  background: rgba(124, 184, 124, 0.1);
  border: 1px solid rgba(124, 184, 124, 0.3);
  border-radius: 12px;
  color: var(--moss-glow);
}

.card-creator {
  color: var(--text-secondary);
}

/* === Empty State === */
.card-empty-state {
  padding: 2rem;
  text-align: center;
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 0.875rem;
  font-style: italic;
}

/* === Profile Card Tagline === */
.card-tagline {
  font-family: var(--font-mono);
  font-size: 0.8rem;
  color: var(--text-secondary);
  margin: 0.25rem 0 0.5rem 0;
  line-height: 1.4;
  font-style: italic;
}

.profile-card--compact .card-tagline {
  font-size: 0.7rem;
  margin: 0.15rem 0 0.35rem 0;
}

/* === Profile Card Connection Info === */
.card-connection-info {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  padding: 0.5rem 0;
  margin-bottom: 0.5rem;
  border-bottom: 1px solid rgba(124, 184, 124, 0.15);
  font-family: var(--font-mono);
  font-size: 0.75rem;
}

.connection-did {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  color: var(--text-muted);
}

.did-label {
  color: var(--text-secondary);
}

.did-value {
  color: var(--cyan);
  font-size: 0.7rem;
  opacity: 0.8;
}

.connection-status-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex-wrap: wrap;
}

.status-indicator {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.15rem 0.5rem;
  border-radius: 12px;
  font-size: 0.65rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.status-indicator::before {
  content: '';
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.status-online {
  background: rgba(124, 184, 124, 0.15);
  color: var(--moss);
}

.status-online::before {
  background: var(--moss);
  box-shadow: 0 0 6px var(--moss);
}

.status-offline {
  background: rgba(255, 100, 100, 0.1);
  color: rgba(255, 100, 100, 0.8);
}

.status-offline::before {
  background: rgba(255, 100, 100, 0.6);
}

.status-unknown {
  background: rgba(150, 150, 150, 0.1);
  color: var(--text-muted);
}

.status-unknown::before {
  background: var(--text-muted);
}

.last-seen {
  color: var(--text-muted);
  font-size: 0.65rem;
}

/* === Compact Profile Card Mode === */
.profile-card--compact .card-header__title {
  font-size: 1rem;
}

.profile-card--compact .card-header__subtitle {
  font-size: 0.7rem;
}

.profile-card--compact .card-connection-info {
  font-size: 0.65rem;
  padding: 0.35rem 0;
  margin-bottom: 0.35rem;
}

.profile-card--compact .did-value {
  font-size: 0.6rem;
}

.profile-card--compact .status-indicator {
  font-size: 0.55rem;
  padding: 0.1rem 0.4rem;
}

.profile-card--compact .last-seen {
  font-size: 0.55rem;
}

/* Compact markdown - smaller text for small cards */
.card-content--compact .card-markdown-section {
  font-size: 0.75rem;
}

.card-content--compact .card-markdown {
  font-size: 0.75rem;
  line-height: 1.5;
}

.card-content--compact .card-markdown h1 {
  font-size: 1rem;
}

.card-content--compact .card-markdown h2 {
  font-size: 0.9rem;
}

.card-content--compact .card-markdown h3 {
  font-size: 0.8rem;
}

.card-content--compact .card-markdown p {
  margin-bottom: 0.5em;
}

.card-content--compact .card-markdown code {
  font-size: 0.7em;
  padding: 1px 4px;
}

.card-content--compact .card-empty-state {
  padding: 1rem;
  font-size: 0.75rem;
}

/* === Responsive Adjustments === */
@media (max-width: 768px) {
  :root {
    --card-base-width: calc(100vw - 2rem);
  }

  /* Keep landscape cards horizontal, just scale them down */
  .golden-card--landscape .golden-card__interior {
    /* Maintain horizontal layout */
    grid-template-columns: var(--split-minor) var(--split-major);
  }

  /* Portrait cards can stack on very small screens */
  .golden-card--portrait .golden-card__interior {
    grid-template-columns: 1fr;
    grid-template-rows: auto 1fr;
  }

  .card-content {
    padding: var(--spacing-phi-md);
  }

  .card-header__title {
    font-size: 0.95rem;
  }

  .card-gallery {
    grid-template-columns: repeat(auto-fill, minmax(40px, 1fr));
  }

  .card-actions {
    flex-direction: column;
  }

  .btn-primary,
  .btn-secondary {
    width: 100%;
  }
}

/* Extra small screens - optimize for mobile */
@media (max-width: 500px) {
  :root {
    --card-base-width: calc(100vw - 1rem);
  }

  .golden-card {
    border-radius: 6px;
  }

  /* Keep landscape layout even on small screens */
  .golden-card--landscape .golden-card__interior {
    grid-template-columns: var(--split-minor) var(--split-major);
  }

  .card-content {
    padding: var(--spacing-phi-sm);
  }

  .card-header__title {
    font-size: 0.85rem;
    line-height: 1.2;
  }

  .card-header__subtitle,
  .card-header__link {
    font-size: var(--text-xs);
  }

  .card-image-area {
    min-height: 120px;
  }

  .card-footer {
    font-size: var(--text-xs);
  }

  /* Adjust markdown content for readability */
  .card-markdown-section {
    font-size: var(--text-xs);
  }

  /* Smaller gallery items */
  .card-gallery {
    grid-template-columns: repeat(auto-fill, minmax(32px, 1fr));
  }
}

/* ═══════════════════════════════════════════════════════════════════
   INTENTION CREATOR - Full-featured creation form
   ═══════════════════════════════════════════════════════════════════ */

.intention-creator-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.85);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: var(--spacing-phi-lg);
}

.intention-creator {
  background: rgba(10, 10, 10, 0.95);
  border: 1px solid var(--color-moss-glow);
  box-shadow: 0 0 30px rgba(124, 184, 124, 0.2),
              inset 0 0 20px rgba(124, 184, 124, 0.05);
  max-width: 700px;
  width: 100%;
  max-height: 90vh;
  overflow-y: auto;
  border-radius: 4px;
}

/* Header */
.creator-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-phi-md);
  border-bottom: 1px solid var(--color-moss);
}

.creator-title {
  font-family: var(--font-serif);
  font-size: 1.5rem;
  font-weight: 300;
  font-style: italic;
  color: var(--color-moss-glow);
  letter-spacing: 0.05em;
  margin: 0;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.creator-icon {
  color: var(--color-cyan);
  font-style: normal;
  font-size: 1.75rem;
}

.creator-close {
  background: none;
  border: none;
  color: var(--color-white-dim);
  font-size: 2rem;
  line-height: 1;
  cursor: pointer;
  padding: 0;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.creator-close:hover {
  color: var(--color-red);
  transform: scale(1.1);
}

/* Form */
.creator-form {
  padding: var(--spacing-phi-md);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-phi-md);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.form-label {
  font-family: var(--font-mono);
  font-size: 0.875rem;
  color: var(--color-moss-glow);
  letter-spacing: 0.05em;
  text-transform: lowercase;
}

.form-input,
.form-textarea {
  font-family: var(--font-mono);
  font-size: 1rem;
  background: rgba(10, 10, 10, 0.8);
  border: 1px solid rgba(90, 122, 90, 0.3);
  color: var(--color-white);
  padding: 0.75rem;
  border-radius: 2px;
  transition: all 0.3s ease;
}

.form-input::placeholder,
.form-textarea::placeholder {
  color: rgba(90, 122, 90, 0.5);
}

.form-input:focus,
.form-textarea:focus {
  outline: none;
  border-color: var(--color-cyan);
  box-shadow: 0 0 10px rgba(0, 212, 170, 0.3);
}

.form-input--title {
  font-size: 1.125rem;
  font-weight: 500;
}

.form-textarea {
  resize: vertical;
  min-height: 120px;
  line-height: 1.6;
}

/* Category buttons */
.category-buttons {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.category-btn {
  font-family: var(--font-mono);
  font-size: 0.875rem;
  padding: 0.5rem 1rem;
  background: rgba(90, 122, 90, 0.1);
  border: 1px solid rgba(90, 122, 90, 0.3);
  color: var(--color-moss-glow);
  border-radius: 2px;
  cursor: pointer;
  transition: all 0.2s ease;
  text-transform: lowercase;
  letter-spacing: 0.05em;
}

.category-btn:hover {
  background: rgba(90, 122, 90, 0.2);
  border-color: var(--color-moss-glow);
}

.category-btn--active {
  background: rgba(0, 212, 170, 0.15);
  border-color: var(--color-cyan);
  color: var(--color-cyan);
  box-shadow: 0 0 10px rgba(0, 212, 170, 0.2);
}

/* Actions */
.creator-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-phi-sm);
  padding: var(--spacing-phi-md);
  border-top: 1px solid var(--color-moss);
}

.btn-cancel,
.btn-manifest {
  font-family: var(--font-mono);
  font-size: 0.875rem;
  padding: 0.75rem 1.5rem;
  border-radius: 2px;
  cursor: pointer;
  transition: all 0.2s ease;
  text-transform: lowercase;
  letter-spacing: 0.05em;
  border: 1px solid;
}

.btn-cancel {
  background: transparent;
  border-color: var(--color-moss);
  color: var(--color-white-dim);
}

.btn-cancel:hover {
  border-color: var(--color-white);
  color: var(--color-white);
}

.btn-manifest {
  background: var(--color-cyan);
  border-color: var(--color-cyan);
  color: var(--color-void);
  font-weight: 500;
}

.btn-manifest:hover:not(:disabled) {
  background: var(--color-neon-green);
  border-color: var(--color-neon-green);
  box-shadow: 0 0 15px rgba(57, 255, 20, 0.4);
  transform: translateY(-1px);
}

.btn-manifest:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Keyboard hint */
.creator-hint {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--color-moss);
  text-align: center;
  padding: var(--spacing-phi-sm) var(--spacing-phi-md);
  border-top: 1px solid rgba(90, 122, 90, 0.2);
}

.hint-key {
  color: var(--color-cyan);
  font-weight: 500;
  padding: 0.125rem 0.375rem;
  background: rgba(0, 212, 170, 0.1);
  border-radius: 2px;
}

/* Mobile adjustments */
@media (max-width: 600px) {
  .intention-creator {
    max-height: 95vh;
  }

  .creator-title {
    font-size: 1.25rem;
  }

  .category-buttons {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
  }

  .category-btn {
    width: 100%;
  }

  .creator-actions {
    flex-direction: column;
  }

  .btn-cancel,
  .btn-manifest {
    width: 100%;
  }
}

/* ═══════════════════════════════════════════════════════════════════
   MARKDOWN EDITOR - Split-pane editor with live preview
   ═══════════════════════════════════════════════════════════════════ */

.markdown-editor {
  width: 100%;
  border: 1px solid rgba(90, 122, 90, 0.3);
  border-radius: 2px;
  background: rgba(10, 10, 10, 0.8);
}

.md-toolbar {
  display: flex;
  gap: 0.5rem;
  padding: 0.5rem;
  background: rgba(90, 122, 90, 0.05);
  border-bottom: 1px solid rgba(90, 122, 90, 0.2);
  flex-wrap: wrap;
}

.md-toolbar-group {
  display: flex;
  gap: 0.25rem;
  padding-right: 0.5rem;
  border-right: 1px solid rgba(90, 122, 90, 0.2);
}

.md-toolbar-group:last-child {
  border-right: none;
}

.md-toolbar-toggle {
  margin-left: auto;
}

.md-btn {
  font-family: var(--font-mono);
  font-size: 0.875rem;
  padding: 0.375rem 0.625rem;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 2px;
  color: var(--color-moss);
  cursor: pointer;
  transition: all 0.2s ease;
  min-width: 32px;
  text-align: center;
}

.md-btn:hover {
  background: rgba(124, 184, 124, 0.1);
  border-color: var(--color-moss);
  color: var(--color-moss-glow);
}

.md-btn--active {
  background: rgba(0, 212, 170, 0.15);
  border-color: var(--color-cyan);
  color: var(--color-cyan);
}

.md-content {
  display: flex;
  gap: 0;
  position: relative;
}

.md-pane {
  padding: 0.75rem;
  overflow-y: auto;
}

.md-pane--editor {
  flex: 1;
  border-right: 1px solid rgba(90, 122, 90, 0.2);
}

.md-pane--preview {
  flex: 1;
  background: rgba(0, 0, 0, 0.2);
}

.md-pane--full {
  flex: 1;
}

.md-textarea {
  width: 100%;
  min-height: 200px;
  font-family: var(--font-mono);
  font-size: 0.875rem;
  background: transparent;
  border: none;
  color: var(--color-white);
  resize: vertical;
  line-height: 1.6;
}

.md-textarea:focus {
  outline: none;
}

.md-textarea::placeholder {
  color: rgba(90, 122, 90, 0.5);
}

.md-preview-empty {
  color: var(--color-moss);
  font-family: var(--font-mono);
  font-size: 0.875rem;
  font-style: italic;
  text-align: center;
  padding: 2rem;
}

.md-help {
  border-top: 1px solid rgba(90, 122, 90, 0.2);
}

.md-help-toggle {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--color-moss);
  padding: 0.5rem 0.75rem;
  cursor: pointer;
  user-select: none;
  transition: color 0.2s ease;
}

.md-help-toggle:hover {
  color: var(--color-cyan);
}

.md-help-content {
  padding: 0.75rem;
  background: rgba(0, 0, 0, 0.3);
}

.md-ref-table {
  width: 100%;
  font-family: var(--font-mono);
  font-size: 0.75rem;
}

.md-ref-table td {
  padding: 0.25rem 0.5rem;
  color: var(--color-white-dim);
}

.md-ref-table td:first-child {
  color: var(--color-cyan);
  font-weight: 500;
}

/* Image upload additions */
.image-upload-container {
  display: flex;
  align-items: center;
  gap: 1rem;
  flex-wrap: wrap;
}

.image-upload-status {
  font-family: var(--font-mono);
  font-size: 0.875rem;
  color: var(--color-moss-glow);
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  background: rgba(124, 184, 124, 0.1);
  border-radius: 2px;
}

.image-remove-btn {
  background: transparent;
  border: none;
  color: var(--color-white-dim);
  font-size: 1.25rem;
  line-height: 1;
  cursor: pointer;
  padding: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color 0.2s ease;
}

.image-remove-btn:hover {
  color: var(--color-red);
}

/* Mobile adjustments for markdown editor */
@media (max-width: 768px) {
  .md-content {
    flex-direction: column;
  }

  .md-pane--editor {
    border-right: none;
    border-bottom: 1px solid rgba(90, 122, 90, 0.2);
  }
}

/* ===============================================
   CONTACT COMPONENTS
   =============================================== */

/* Generate Invite Button */
.generate-invite-button {
  width: 280px;
  height: 48px;
  margin: 24px auto;
  border: 2px solid var(--gold);
  background: transparent;
  color: var(--gold);
  font-family: var(--font-serif);
  font-size: 18px;
  cursor: pointer;
  transition: all 300ms ease;
}

.generate-invite-button:hover {
  background: rgba(212, 175, 55, 0.1);
  box-shadow: 0 0 16px rgba(212, 175, 55, 0.4);
}

.generate-invite-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Invite QR Overlay */
/* Invite Overlay (text-only, no QR) */
.invite-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.9);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fade-in 300ms ease-out;
}

.invite-content {
  background: var(--void-black);
  border: 2px solid var(--gold);
  border-radius: 12px;
  padding: 2.5rem;
  max-width: 600px;
  width: 90%;
  box-shadow: 0 0 60px rgba(212, 175, 55, 0.4);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1.5rem;
  animation: modal-appear 300ms ease-out;
}

.overlay-title {
  font-family: var(--font-serif);
  font-size: 2rem;
  font-style: italic;
  color: var(--gold);
  margin: 0;
  letter-spacing: 0.05em;
}

.overlay-description {
  font-family: var(--font-mono);
  font-size: var(--text-base);
  color: var(--text-secondary);
  text-align: center;
  margin: 0;
  max-width: 400px;
}

.invite-code-display {
  width: 100%;
  background: rgba(0, 212, 170, 0.05);
  border: 2px solid var(--cyan);
  border-radius: 8px;
  padding: 1.5rem;
}

.invite-code-text {
  font-family: var(--font-mono);
  font-size: 16px;
  line-height: 1.6;
  color: var(--cyan);
  margin: 0;
  word-break: break-all;
  white-space: pre-wrap;
  text-align: center;
}

.copy-code-button {
  width: 180px;
  height: 48px;
  border: 2px solid var(--moss);
  background: transparent;
  color: var(--moss-glow);
  font-family: var(--font-mono);
  font-size: 16px;
  cursor: pointer;
  transition: all 300ms ease;
  border-radius: 4px;
}

.copy-code-button:hover {
  background: rgba(124, 184, 124, 0.1);
  box-shadow: 0 0 16px rgba(124, 184, 124, 0.4);
  transform: translateY(-2px);
}

.close-overlay-button {
  width: 120px;
  height: 40px;
  border: 1px solid var(--text-muted);
  background: transparent;
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 14px;
  cursor: pointer;
  transition: all 300ms ease;
  border-radius: 4px;
}

.close-overlay-button:hover {
  border-color: var(--gold);
  color: var(--gold);
}

/* Modal Overlay */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.85);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fade-in 300ms ease-out;
}

.invite-code-modal,
.inviter-preview-modal {
  background: var(--void-black);
  border: 2px solid var(--gold);
  padding: 40px;
  max-width: 500px;
  width: 90%;
  box-shadow: 0 0 40px rgba(212, 175, 55, 0.3);
}

/* Profile Edit Modal */
.profile-edit-modal {
  background: var(--void-black);
  border: 2px solid var(--gold);
  padding: 40px;
  max-width: 800px;
  width: 90%;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: 0 0 40px rgba(212, 175, 55, 0.3);
  position: relative;
  animation: modal-appear 300ms ease-out;
}

.modal-close-btn {
  position: absolute;
  top: 20px;
  right: 20px;
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: 32px;
  line-height: 1;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 200ms ease;
  z-index: 10;
}

.modal-close-btn:hover {
  color: var(--gold);
  transform: rotate(90deg);
}

@keyframes modal-appear {
  from {
    opacity: 0;
    transform: scale(0.95) translateY(-20px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

.modal-title {
  font-family: var(--font-serif);
  font-size: 28px;
  color: var(--gold);
  margin-bottom: 24px;
  text-align: center;
}

.modal-description {
  font-family: var(--font-mono);
  font-size: 14px;
  color: var(--text-secondary);
  margin-bottom: 20px;
  text-align: center;
}

.invite-input {
  width: 100%;
  height: 48px;
  background: rgba(15, 15, 15, 0.8);
  border: 2px solid rgba(212, 175, 55, 0.3);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 14px;
  padding: 0 16px;
  margin-bottom: 16px;
  transition: all 300ms ease;
}

.invite-input:focus {
  outline: none;
  border-color: var(--cyan);
  box-shadow: 0 0 12px rgba(0, 212, 170, 0.2);
}

.invite-input.invalid {
  border-color: var(--danger);
}

.invite-input::placeholder {
  color: var(--text-muted);
}

.error-text {
  font-family: var(--font-mono);
  font-size: 13px;
  color: var(--danger);
  margin-bottom: 16px;
}

.decode-button,
.cancel-button {
  width: 100%;
  height: 48px;
  margin-top: 12px;
  border: 2px solid var(--cyan);
  background: transparent;
  color: var(--cyan);
  font-family: var(--font-mono);
  font-size: 16px;
  cursor: pointer;
  transition: all 300ms ease;
}

.decode-button:hover {
  background: rgba(0, 212, 170, 0.1);
  box-shadow: 0 0 16px rgba(0, 212, 170, 0.3);
}

.decode-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.cancel-button {
  border-color: var(--text-muted);
  color: var(--text-muted);
}

.cancel-button:hover {
  border-color: var(--text-primary);
  color: var(--text-primary);
  background: rgba(245, 245, 245, 0.05);
}

/* Profile Preview in Modal */
.profile-preview {
  margin: 24px 0;
  padding: 20px;
  background: rgba(15, 15, 15, 0.6);
  border: 1px solid rgba(212, 175, 55, 0.2);
  border-radius: 4px;
}

.profile-header {
  margin-bottom: 12px;
}

.profile-header h3 {
  font-family: var(--font-serif);
  font-size: 22px;
  color: var(--gold);
  margin-bottom: 4px;
}

.profile-subtitle {
  font-family: var(--font-mono);
  font-size: 14px;
  color: var(--cyan);
  font-style: italic;
}

.profile-bio {
  font-family: var(--font-mono);
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1.6;
}

.request-message {
  font-family: var(--font-mono);
  font-size: 14px;
  color: var(--moss-glow);
  text-align: center;
  margin: 20px 0;
}

.modal-actions {
  display: flex;
  gap: 16px;
  margin-top: 24px;
}

/* Accept button (primary action) */
.accept-button {
  flex: 1;
  height: 56px;
  border: 3px solid var(--gold);
  background: linear-gradient(135deg, rgba(212, 175, 55, 0.1) 0%, rgba(212, 175, 55, 0.05) 100%);
  color: var(--gold);
  font-family: var(--font-serif);
  font-size: 20px;
  letter-spacing: 1px;
  cursor: pointer;
  transition: all 300ms ease;
}

.accept-button:hover {
  background: rgba(212, 175, 55, 0.15);
  box-shadow: 0 0 20px rgba(212, 175, 55, 0.3), inset 0 0 12px rgba(212, 175, 55, 0.1);
  transform: translateY(-2px);
}

/* Decline button */
.decline-button {
  flex: 1;
  height: 56px;
  border: 2px solid var(--danger);
  background: transparent;
  color: var(--danger);
  font-family: var(--font-mono);
  font-size: 16px;
  cursor: pointer;
  transition: all 300ms ease;
}

.decline-button:hover {
  background: rgba(255, 51, 102, 0.1);
  box-shadow: 0 0 16px rgba(255, 51, 102, 0.3);
}

/* Pending Requests Section */
.pending-requests-section {
  margin: 2rem 0;
  padding: 1.5rem;
  background: rgba(15, 15, 15, 0.6);
  border: 1px solid rgba(212, 175, 55, 0.2);
  border-radius: 8px;
}

.pending-requests-section .section-title {
  font-family: var(--font-serif);
  font-size: 24px;
  color: var(--gold);
  margin-bottom: 20px;
}

.incoming-requests,
.outgoing-requests {
  margin-bottom: 24px;
}

.incoming-requests h4,
.outgoing-requests h4 {
  font-family: var(--font-mono);
  font-size: 14px;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 1px;
  margin-bottom: 12px;
}

.pending-card {
  padding: 16px;
  margin-bottom: 12px;
  background: rgba(212, 175, 55, 0.05);
  border-left: 3px solid var(--gold);
  border-radius: 4px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  animation: pulse-glow 3s infinite;
}

.pending-card.incoming {
  border-left-color: var(--cyan);
  background: rgba(0, 212, 170, 0.05);
}

.pending-card.outgoing {
  border-left-color: var(--moss);
  background: rgba(124, 184, 124, 0.05);
  opacity: 0.7;
}

.pending-card span {
  font-family: var(--font-mono);
  font-size: 14px;
  color: var(--text-primary);
}

.pending-actions {
  display: flex;
  gap: 12px;
}

.pending-actions .accept-button,
.pending-actions .decline-button {
  flex: 0;
  width: auto;
  height: 36px;
  padding: 0 20px;
  font-size: 14px;
}

/* Contacts Gallery */
.contacts-gallery,
.contacts-gallery-empty {
  margin: 3rem 0;
}

.contacts-gallery .section-title {
  font-family: var(--font-serif);
  font-size: 28px;
  color: var(--gold);
  margin-bottom: 24px;
}

.empty-state {
  text-align: center;
  padding: 60px 20px;
}

.empty-icon {
  font-size: 64px;
  color: var(--moss);
  margin-bottom: 20px;
  opacity: 0.3;
}

.empty-state p {
  font-family: var(--font-mono);
  font-size: 16px;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.empty-hint {
  font-size: 14px;
  color: var(--text-muted);
  font-style: italic;
}

.contact-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 2rem;
  margin-top: 1.5rem;
}

/* Contact Card */
.contact-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  cursor: pointer;
  transition: all 300ms ease;
  opacity: 0;
  animation: show-contact 600ms ease-out forwards;
  animation-delay: calc(var(--index) * 100ms);
}

.contact-card:hover {
  transform: translateY(-4px);
}

.contact-avatar {
  width: 100px;
  aspect-ratio: 1000 / 1618;
  position: relative;
  overflow: hidden;
  background: var(--void-black);
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 12px;
}

.contact-avatar .avatar-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.avatar-placeholder {
  font-family: var(--font-serif);
  font-size: 48px;
  color: var(--gold);
  text-transform: uppercase;
}

/* Status dot - positioned at bottom right of portrait avatar */
.contact-avatar .status-dot {
  position: absolute;
  bottom: 6px;
  right: 6px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  border: 2px solid var(--void-black);
  background: rgba(255, 255, 255, 0.3);
}

.contact-card.online .status-dot {
  background: var(--cyan);
  animation: pulse-online 2s infinite;
}

.contact-card.offline .status-dot {
  background: rgba(255, 255, 255, 0.3);
}

.contact-name {
  font-family: var(--font-mono);
  font-size: 14px;
  color: var(--text-primary);
  text-align: center;
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Animations */
@keyframes fade-in {
  0% {
    opacity: 0;
  }
  100% {
    opacity: 1;
  }
}

@keyframes pulse-glow {
  0%, 100% {
    box-shadow: 0 0 0 0 rgba(212, 175, 55, 0.2);
  }
  50% {
    box-shadow: 0 0 8px 2px rgba(212, 175, 55, 0.1);
  }
}

@keyframes pulse-online {
  0%, 100% {
    box-shadow: 0 0 0 0 rgba(0, 212, 170, 0.7);
  }
  50% {
    box-shadow: 0 0 0 8px rgba(0, 212, 170, 0);
  }
}

@keyframes show-contact {
  0% {
    opacity: 0;
    transform: translateY(20px) scale(0.9);
  }
  100% {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .contact-grid {
    grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
    gap: 1.5rem;
  }

  .contact-avatar {
    width: 80px;
    /* height auto via aspect-ratio */
  }

  .avatar-placeholder {
    font-size: 36px;
  }

  .modal-actions {
    flex-direction: column;
  }

  .accept-button,
  .decline-button {
    width: 100%;
  }

  .generate-invite-button {
    width: 100%;
  }

  /* Contact Command Center responsive */
  .command-panels {
    flex-direction: column;
  }

  .identity-beacon-panel {
    width: 100%;
    margin-bottom: 1.5rem;
  }

  .hero-actions {
    flex-direction: column;
  }

  .action-card {
    width: 100%;
  }
}

/* === PROFILE PAGE === */
/* Single-column layout with profile as hero */

.profile-page {
  min-height: 100vh;
  background: var(--void-black);
  padding: 1.5rem;
  position: relative;
}

/* Sacred geometry background (subtle) */
.profile-page::before {
  content: '';
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 800px;
  height: 800px;
  background-image: url('assets/seed-of-life.svg');
  background-repeat: no-repeat;
  background-position: center;
  background-size: contain;
  opacity: 0.03;
  pointer-events: none;
  z-index: 0;
}

/* === Header === */
.profile-header {
  display: flex;
  align-items: center;
  gap: 1.5rem;
  margin-bottom: 2.5rem;
  padding-bottom: 1rem;
  border-bottom: 1px solid var(--void-border);
  position: relative;
  z-index: 1;
}

.back-link {
  background: transparent;
  border: 1px solid var(--void-border);
  border-radius: 4px;
  padding: 0.5rem 1rem;
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: all var(--transition-normal);
  text-decoration: none;
}

.back-link:hover {
  border-color: var(--moss);
  color: var(--moss-glow);
}

.profile-title {
  font-family: var(--font-serif);
  font-size: var(--text-2xl);
  font-weight: 400;
  font-style: italic;
  color: var(--gold);
  letter-spacing: 0.05em;
  margin: 0;
}

/* === Loading State === */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1.5rem;
  min-height: 60vh;
  position: relative;
  z-index: 1;
}

.loading-orb {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--moss-glow);
  box-shadow: 0 0 20px var(--moss-glow);
  animation: pulse 2s ease-in-out infinite;
}

.loading-state p {
  font-family: var(--font-mono);
  font-size: var(--text-base);
  font-style: italic;
  color: var(--text-secondary);
}

/* === Main Content === */
.profile-content {
  position: relative;
  z-index: 1;
}

/* === HERO: Identity Card === */
.identity-hero {
  margin-bottom: 2rem;
}

.identity-card {
  display: grid;
  grid-template-columns: 31.8% 1fr;
  gap: 2.5rem;
  align-items: start;
  position: relative;
  animation: heroAppear 0.6s ease-out;
}

/* Left column: Avatar with QR overlay */
.hero-left {
  display: flex;
  flex-direction: column;
  align-items: center;
}

@keyframes heroAppear {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.hero-avatar {
  position: relative;
  width: 100%;
  aspect-ratio: 1000 / 1618;
  background: var(--void-black);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.hero-avatar-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

/* QR code overlay at bottom of avatar - 31.8% width (golden ratio minor) */
.avatar-qr-overlay {
  position: absolute;
  bottom: 1rem;
  left: 50%;
  transform: translateX(-50%);
  width: 31.8%;
  background: rgba(10, 10, 10, 0.7);
  padding: 4px;
  border: 1px solid var(--gold);
  border-radius: 2px;
  box-shadow:
    0 0 8px var(--gold-glow),
    0 0 16px var(--gold-glow),
    inset 0 0 4px rgba(212, 175, 55, 0.1);
  backdrop-filter: blur(4px);
}

.avatar-qr-overlay .qr-signature {
  width: 100%;
}

.avatar-qr-overlay .qr-signature svg {
  width: 100%;
  height: auto;
}

/* Connection action buttons under avatar */
.connection-actions {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-top: 1rem;
  width: 100%;
}

.connection-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.6rem 0.75rem;
  background: transparent;
  border: 1px solid var(--gold);
  border-radius: 4px;
  color: var(--gold);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  cursor: pointer;
  transition: all var(--transition-normal);
  position: relative;
  overflow: hidden;
  width: 100%;
}

.connection-btn:hover {
  background: rgba(212, 175, 55, 0.1);
  box-shadow: 0 0 12px var(--gold-glow);
}

.connection-btn .btn-icon {
  font-size: 1rem;
  transition: transform var(--transition-normal);
  flex-shrink: 0;
}

.connection-btn .btn-text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Invite button - paper airplane fly animation on click */
.invite-btn.copied {
  background: rgba(212, 175, 55, 0.15);
  border-color: var(--gold);
  box-shadow: 0 0 16px var(--gold-glow);
}

.invite-btn.copied .btn-icon {
  animation: flyAway 0.5s ease-out forwards;
}

@keyframes flyAway {
  0% {
    transform: translate(0, 0) rotate(0deg);
    opacity: 1;
  }
  50% {
    transform: translate(10px, -10px) rotate(15deg);
    opacity: 0.8;
  }
  100% {
    transform: translate(20px, -20px) rotate(25deg);
    opacity: 0;
  }
}

/* Scan button */
.scan-btn:hover .btn-icon {
  animation: pulse 0.6s ease-in-out infinite;
}

@keyframes cameraFlash {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.hero-identity {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  flex: 1;
}

.hero-name {
  font-family: var(--font-serif);
  font-size: 2.5rem;
  font-weight: 400;
  color: var(--gold);
  margin: 0;
  letter-spacing: 0.02em;
}

.hero-subtitle {
  font-family: var(--font-sans);
  font-size: var(--text-lg);
  font-style: italic;
  color: var(--cyan);
  margin: 0;
}

.hero-bio {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-secondary);
  line-height: 1.6;
}

.hero-did {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  color: var(--text-muted);
  margin-top: 0.5rem;
  word-break: break-all;
}

/* === Inline Editing Styles === */

/* Editable fields */
.editable {
  cursor: pointer;
  transition: all var(--transition-fast);
  border-radius: 4px;
  padding: 0.25rem 0.5rem;
  margin: -0.25rem -0.5rem;
}

.editable:hover {
  background: rgba(212, 175, 55, 0.08);
  box-shadow: inset 0 0 0 1px rgba(212, 175, 55, 0.3);
}

.editable.placeholder {
  color: var(--text-muted);
  font-style: italic;
}

.editable.placeholder:hover {
  color: var(--text-secondary);
}

/* Inline edit inputs */
.inline-edit-input,
.inline-edit-textarea {
  width: 100%;
  background: rgba(0, 0, 0, 0.4);
  border: 2px solid var(--gold);
  border-radius: 4px;
  color: var(--text-primary);
  font-family: inherit;
  transition: all var(--transition-fast);
  outline: none;
}

.inline-edit-input:focus,
.inline-edit-textarea:focus {
  border-color: var(--cyan);
  box-shadow: 0 0 16px rgba(0, 212, 170, 0.3);
}

.inline-edit-input.name-input {
  font-family: var(--font-serif);
  font-size: 2.5rem;
  font-weight: 400;
  color: var(--gold);
  padding: 0.25rem 0.5rem;
}

.inline-edit-input.subtitle-input {
  font-family: var(--font-sans);
  font-size: var(--text-lg);
  font-style: italic;
  color: var(--cyan);
  padding: 0.5rem;
}

.inline-edit-textarea.bio-input {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-secondary);
  line-height: 1.6;
  padding: 0.75rem;
  min-height: 120px;
  resize: vertical;
}

/* Bio edit container with save/cancel buttons */
.bio-edit-container {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.bio-edit-actions {
  display: flex;
  gap: 0.75rem;
}

.bio-save-btn {
  padding: 0.5rem 1.25rem;
  background: rgba(0, 212, 170, 0.15);
  border: 1px solid var(--cyan);
  color: var(--cyan);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.bio-save-btn:hover {
  background: rgba(0, 212, 170, 0.25);
  box-shadow: 0 0 12px rgba(0, 212, 170, 0.3);
}

.bio-cancel-btn {
  padding: 0.5rem 1.25rem;
  background: transparent;
  border: 1px solid var(--text-muted);
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.bio-cancel-btn:hover {
  border-color: var(--text-secondary);
  color: var(--text-secondary);
}

/* === Contact Exchange Section === */
.contact-exchange {
  margin-bottom: 2.5rem;
}

.section-title {
  font-family: var(--font-serif);
  font-size: var(--text-xl);
  font-style: italic;
  color: var(--gold);
  margin: 0 0 1.5rem 0;
  text-align: center;
}

.exchange-actions {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1.5rem;
}

.exchange-card {
  background: rgba(15, 15, 15, 0.6);
  border: 1px solid var(--void-border);
  border-radius: 8px;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  transition: all var(--transition-normal);
}

.exchange-card:hover {
  border-color: var(--cyan);
  box-shadow: 0 0 20px rgba(0, 212, 170, 0.15);
}

.card-icon {
  font-size: 2.5rem;
  color: var(--gold);
}

.card-title {
  font-family: var(--font-serif);
  font-size: var(--text-lg);
  color: var(--gold);
  margin: 0;
}

.card-description {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-secondary);
  text-align: center;
  margin: 0;
}

.exchange-btn {
  padding: 0.75rem 1.5rem;
  background: transparent;
  border: 2px solid var(--cyan);
  color: var(--cyan);
  font-family: var(--font-mono);
  font-size: var(--text-base);
  cursor: pointer;
  transition: all var(--transition-normal);
  border-radius: 4px;
  margin-top: 0.5rem;
}

.exchange-btn:hover {
  background: rgba(0, 212, 170, 0.1);
  box-shadow: 0 0 12px rgba(0, 212, 170, 0.3);
}

/* === Contacts Section === */
.contacts-section {
  margin-top: 1rem;
}

/* === Error State === */
.error-state {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 60vh;
  position: relative;
  z-index: 1;
}

.error-state p {
  font-family: var(--font-mono);
  font-size: var(--text-base);
  color: var(--danger);
  padding: 1rem 2rem;
  border: 1px solid var(--danger);
  border-radius: 4px;
  background: rgba(255, 51, 102, 0.1);
}

/* === Enhanced QR Overlay for Generate Invite === */
.invite-qr-overlay {
  position: fixed;
  inset: 0;
  background: rgba(10, 10, 10, 0.92);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.invite-qr-content {
  background: var(--void-lighter);
  border: 2px solid var(--gold);
  border-radius: 12px;
  padding: 2.5rem;
  max-width: 500px;
  width: 90%;
  position: relative;
  box-shadow: 0 0 60px rgba(212, 175, 55, 0.3);
  animation: scaleIn 0.3s ease;
}

@keyframes scaleIn {
  from {
    transform: scale(0.9);
    opacity: 0;
  }
  to {
    transform: scale(1);
    opacity: 1;
  }
}

.overlay-title {
  font-family: var(--font-serif);
  font-size: var(--text-2xl);
  font-weight: 400;
  font-style: italic;
  color: var(--gold);
  text-align: center;
  margin: 0 0 2rem 0;
}

.qr-code-container {
  display: flex;
  justify-content: center;
  padding: 1.5rem;
  background: white;
  border-radius: 8px;
  margin-bottom: 1.5rem;
}

.invite-code-display {
  margin-bottom: 1.5rem;
}

.invite-label {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  color: var(--text-secondary);
  margin: 0 0 0.5rem 0;
  text-transform: lowercase;
}

.invite-code-text {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--cyan);
  background: var(--void-black);
  padding: 1rem;
  border-radius: 4px;
  border: 1px solid var(--void-border);
  margin: 0 0 1rem 0;
  overflow-x: auto;
  word-break: break-all;
}

.copy-code-button {
  width: 100%;
  padding: 0.75rem;
  background: transparent;
  border: 1px solid var(--cyan);
  border-radius: 4px;
  color: var(--cyan);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: all var(--transition-normal);
}

.copy-code-button:hover {
  background: rgba(0, 212, 170, 0.1);
  box-shadow: 0 0 16px var(--cyan-glow);
}

.dismiss-qr-button {
  width: 100%;
  padding: 0.75rem;
  background: transparent;
  border: 1px solid var(--text-muted);
  border-radius: 4px;
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: all var(--transition-normal);
}

.dismiss-qr-button:hover {
  border-color: var(--text-primary);
  color: var(--text-primary);
}

/* === Enhanced Pending Requests === */
.pending-requests-section {
  background: var(--void-lighter);
  border: 1px solid var(--void-border);
  border-radius: 8px;
  padding: 1.5rem;
}

.pending-requests-section .section-title {
  font-family: var(--font-serif);
  font-size: var(--text-xl);
  font-weight: 400;
  font-style: italic;
  color: var(--gold);
  margin: 0 0 1rem 0;
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

/* Pulsing notification dot */
.pending-requests-section .section-title::before {
  content: '';
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--gold);
  box-shadow: 0 0 12px var(--gold-glow);
  animation: pulse 2s ease-in-out infinite;
}

.subsection-title {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-secondary);
  margin: 0 0 0.75rem 0;
  text-transform: lowercase;
}

.pending-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem;
  background: var(--void-black);
  border: 1px solid var(--void-border);
  border-radius: 4px;
  margin-bottom: 0.75rem;
  transition: all var(--transition-normal);
}

.pending-card:hover {
  border-color: var(--moss);
}

.pending-card.incoming {
  border-left: 3px solid var(--gold);
}

.pending-card.outgoing {
  border-left: 3px solid var(--cyan);
}

.pending-info {
  flex: 1;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
}

.pending-name {
  color: var(--cyan);
  font-weight: 500;
}

.pending-message {
  color: var(--text-secondary);
}

.pending-actions {
  display: flex;
  gap: 0.5rem;
}

.btn-small {
  padding: 0.5rem 1rem;
  background: transparent;
  border: 1px solid var(--void-border);
  border-radius: 4px;
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  cursor: pointer;
  transition: all var(--transition-normal);
}

.accept-button {
  color: var(--moss-glow);
  border-color: var(--moss-glow);
}

.accept-button:hover {
  background: rgba(124, 184, 124, 0.1);
  box-shadow: 0 0 12px rgba(124, 184, 124, 0.2);
}

.decline-button,
.cancel-button {
  color: var(--text-muted);
}

.decline-button:hover,
.cancel-button:hover {
  color: var(--danger);
  border-color: var(--danger);
}

/* === Enhanced Contacts Gallery === */
.contacts-gallery {
  background: var(--void-lighter);
  border: 1px solid var(--void-border);
  border-radius: 8px;
  padding: 1.5rem;
}

.contacts-gallery .section-title {
  font-family: var(--font-serif);
  font-size: var(--text-xl);
  font-weight: 400;
  font-style: italic;
  color: var(--gold);
  margin: 0 0 1.5rem 0;
}

.contacts-gallery-empty {
  background: var(--void-lighter);
  border: 1px solid var(--void-border);
  border-radius: 8px;
  padding: 1.5rem;
}

.contacts-gallery-empty .section-title {
  font-family: var(--font-serif);
  font-size: var(--text-xl);
  font-weight: 400;
  font-style: italic;
  color: var(--gold);
  margin: 0 0 1.5rem 0;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  padding: 3rem 2rem;
  text-align: center;
}

.empty-icon {
  font-size: 3rem;
  color: var(--text-muted);
  opacity: 0.5;
}

.empty-state p {
  font-family: var(--font-mono);
  font-size: var(--text-base);
  color: var(--text-secondary);
  margin: 0;
}

.empty-hint {
  font-size: var(--text-sm);
  color: var(--text-muted);
  font-style: italic;
}

/* ═══════════════════════════════════════════════════════════════════════
   NETWORK PAGE - FIELD TOPOLOGY
   ═══════════════════════════════════════════════════════════════════════ */

.network-page {
  min-height: 100vh;
  background: var(--void-black);
  padding: 2rem;
}

.network-header {
  display: flex;
  align-items: center;
  gap: 2rem;
  margin-bottom: 2rem;
}

.network-title {
  font-family: var(--font-serif);
  font-size: var(--text-3xl);
  font-weight: 400;
  color: var(--gold);
  text-shadow: 0 0 30px var(--gold-glow);
  letter-spacing: 0.1em;
}

.network-content {
  max-width: 1000px;
  margin: 0 auto;
}

/* === Stats Cards === */
.network-stats {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1.5rem;
  margin-bottom: 3rem;
}

.stat-card {
  background: rgba(15, 15, 15, 0.6);
  border: 1px solid rgba(212, 175, 55, 0.2);
  border-radius: 8px;
  padding: 1.5rem;
  text-align: center;
  backdrop-filter: blur(12px);
  transition: all var(--transition-normal);
}

.stat-card:hover {
  border-color: rgba(212, 175, 55, 0.4);
  box-shadow: 0 0 20px var(--gold-glow);
}

.stat-card .stat-value {
  font-size: var(--text-3xl);
  font-weight: 600;
  color: var(--gold);
  margin-bottom: 0.25rem;
  font-family: var(--font-mono);
}

.stat-card .stat-label {
  font-size: var(--text-lg);
  color: var(--text-primary);
  font-family: var(--font-serif);
  margin-bottom: 0.25rem;
}

.stat-card .stat-sublabel {
  font-size: var(--text-sm);
  color: var(--text-muted);
  font-family: var(--font-mono);
}

/* === Network Sections === */
.network-section {
  margin-bottom: 3rem;
}

.network-section .section-title {
  font-family: var(--font-serif);
  font-size: var(--text-xl);
  font-weight: 400;
  color: var(--gold);
  margin-bottom: 0.25rem;
}

.network-section .section-subtitle {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-muted);
  margin-bottom: 1.5rem;
}

/* === Network Toolbar === */
.network-toolbar {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.75rem 1.5rem;
  background: rgba(15, 15, 15, 0.3);
  border-bottom: 1px solid var(--void-border);
  margin-bottom: 1.5rem;
}

.sync-btn {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  padding: 0.5rem 1rem;
  background: linear-gradient(135deg, rgba(0, 212, 170, 0.1) 0%, rgba(0, 212, 170, 0.05) 100%);
  border: 1px solid rgba(0, 212, 170, 0.3);
  border-radius: 6px;
  color: var(--cyan);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.sync-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, rgba(0, 212, 170, 0.2) 0%, rgba(0, 212, 170, 0.1) 100%);
  border-color: var(--cyan);
  box-shadow: 0 0 12px rgba(0, 212, 170, 0.2);
}

.sync-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.sync-btn.syncing {
  border-color: var(--gold);
  color: var(--gold);
  background: linear-gradient(135deg, rgba(212, 175, 55, 0.1) 0%, rgba(212, 175, 55, 0.05) 100%);
}

.sync-btn .sync-icon {
  font-size: var(--text-lg);
  line-height: 1;
}

.sync-btn.syncing .sync-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.sync-result {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--moss);
  opacity: 0.9;
}

/* === Network Cards === */
.peer-list,
.pinner-list,
.pin-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.network-card {
  background: rgba(15, 15, 15, 0.4);
  border: 1px solid var(--void-border);
  border-radius: 8px;
  padding: 1rem 1.25rem;
  transition: all var(--transition-fast);
}

.network-card:hover {
  border-color: rgba(0, 212, 170, 0.3);
  background: rgba(15, 15, 15, 0.6);
}

.network-card .card-header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 0.5rem;
}

.network-card .status-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}

.network-card .status-dot.status-online {
  background: var(--moss-glow);
  box-shadow: 0 0 10px var(--moss-glow);
  animation: pulse 2s ease-in-out infinite;
}

.network-card .status-dot.status-offline {
  background: var(--text-muted);
}

.network-card .card-name {
  font-family: var(--font-mono);
  font-size: var(--text-base);
  color: var(--text-primary);
  flex-grow: 1;
}

.network-card .card-status {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--moss);
}

.network-card .card-relationship {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--cyan);
  margin-left: auto;
}

.network-card .card-details {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
  align-items: center;
  padding-left: 1.5rem;
}

.network-card .detail-item {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
}

.network-card .detail-label {
  color: var(--text-muted);
}

.network-card .detail-value {
  color: var(--text-secondary);
}

/* Unpin button in YourPinCard */
.network-card .unpin-btn {
  margin-left: auto;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  padding: 0.25rem 0.75rem;
  background: transparent;
  border: 1px solid rgba(255, 51, 102, 0.3);
  border-radius: 4px;
  color: var(--danger);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.network-card .unpin-btn:hover {
  background: rgba(255, 51, 102, 0.1);
  border-color: var(--danger);
}

/* Contact card specific styles */
.network-card .favorite-badge {
  color: var(--gold);
  font-size: var(--text-sm);
  margin-left: 0.25rem;
}

.network-card .did-value {
  color: var(--cyan);
  font-family: var(--font-mono);
}

/* ═══════════════════════════════════════════════════════════════════════════
   MIRRORED PROFILES - Full Profile Cards for My Network
   ═══════════════════════════════════════════════════════════════════════════ */

.mirrored-profiles-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 1.5rem;
}

.mirrored-profile-card {
  background: linear-gradient(
    135deg,
    rgba(15, 15, 15, 0.6) 0%,
    rgba(20, 20, 25, 0.4) 100%
  );
  border: 1px solid var(--void-border);
  border-radius: 12px;
  padding: 1.5rem;
  transition: all var(--transition-normal);
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.mirrored-profile-card:hover {
  border-color: rgba(0, 212, 170, 0.4);
  background: linear-gradient(
    135deg,
    rgba(20, 20, 20, 0.7) 0%,
    rgba(25, 25, 30, 0.5) 100%
  );
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
}

/* Profile header with avatar */
.mirrored-profile-header {
  display: flex;
  align-items: flex-start;
  gap: 1rem;
  position: relative;
}

.mirrored-avatar {
  flex-shrink: 0;
  width: 64px;
  height: 64px;
  border-radius: 50%;
  overflow: hidden;
  border: 2px solid var(--void-border);
  background: rgba(0, 0, 0, 0.3);
}

.mirrored-avatar-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.mirrored-avatar-default {
  opacity: 0.6;
}

.mirrored-identity {
  flex-grow: 1;
  min-width: 0;
}

.mirrored-name {
  font-family: var(--font-serif);
  font-size: var(--text-lg);
  font-weight: 400;
  color: var(--text-primary);
  margin: 0 0 0.25rem 0;
  line-height: 1.2;
}

.mirrored-subtitle {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-secondary);
  margin: 0 0 0.25rem 0;
}

.mirrored-link {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  color: var(--cyan);
  opacity: 0.8;
}

.mirrored-relationship-badge {
  position: absolute;
  top: 0;
  right: 0;
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  color: var(--gold);
  background: rgba(212, 175, 55, 0.1);
  border: 1px solid rgba(212, 175, 55, 0.3);
  border-radius: 4px;
  padding: 0.2rem 0.5rem;
}

/* DID display */
.mirrored-did {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  padding: 0.5rem 0.75rem;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 4px;
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.mirrored-did .did-label {
  color: var(--text-muted);
}

.mirrored-did .did-value {
  color: var(--cyan);
  word-break: break-all;
}

/* Bio section */
.mirrored-bio {
  flex-grow: 1;
}

.mirrored-bio .bio-text {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-secondary);
  line-height: 1.6;
  margin: 0;
}

.mirrored-bio .bio-preview {
  color: var(--text-muted);
}

.mirrored-bio .bio-full {
  white-space: pre-wrap;
}

.mirrored-bio-empty .bio-placeholder {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-muted);
  font-style: italic;
  opacity: 0.6;
  margin: 0;
}

.bio-expand-btn {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  color: var(--cyan);
  background: transparent;
  border: none;
  padding: 0.25rem 0;
  cursor: pointer;
  margin-top: 0.5rem;
  opacity: 0.8;
  transition: opacity var(--transition-fast);
}

.bio-expand-btn:hover {
  opacity: 1;
  text-decoration: underline;
}

/* Timestamps section */
.mirrored-timestamps {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
  padding-top: 0.75rem;
  border-top: 1px solid var(--void-border);
}

.mirrored-timestamps .timestamp-item {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.mirrored-timestamps .timestamp-label {
  color: var(--text-muted);
}

.mirrored-timestamps .timestamp-value {
  color: var(--text-secondary);
}

.mirrored-timestamps .timestamp-mirrored {
  color: var(--moss);
}

/* Actions */
.mirrored-actions {
  display: flex;
  justify-content: flex-end;
  padding-top: 0.5rem;
}

.mirrored-actions .unpin-btn {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  padding: 0.4rem 1rem;
  background: transparent;
  border: 1px solid rgba(255, 51, 102, 0.3);
  border-radius: 4px;
  color: var(--danger);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.mirrored-actions .unpin-btn:hover {
  background: rgba(255, 51, 102, 0.1);
  border-color: var(--danger);
}

/* Responsive mirrored profiles */
@media (max-width: 700px) {
  .mirrored-profiles-grid {
    grid-template-columns: 1fr;
    gap: 1rem;
  }

  .mirrored-profile-card {
    padding: 1rem;
  }

  .mirrored-avatar {
    width: 48px;
    height: 48px;
  }

  .mirrored-timestamps {
    flex-direction: column;
    gap: 0.5rem;
  }
}

/* Empty states */
.network-section .empty-state {
  background: rgba(15, 15, 15, 0.3);
  border: 1px dashed var(--void-border);
  border-radius: 8px;
  padding: 2rem;
  text-align: center;
}

.network-section .empty-state p {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-muted);
  margin: 0;
}

/* === Responsive === */
@media (max-width: 700px) {
  .network-page {
    padding: 1rem;
  }

  .network-stats {
    grid-template-columns: 1fr;
    gap: 1rem;
  }

  .network-card .card-details {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.5rem;
  }

  .network-card .unpin-btn {
    margin-left: 0;
    margin-top: 0.5rem;
  }
}

/* ═══════════════════════════════════════════════════════════════════════════
   SACRED NAVIGATION CONSOLE - Unified Header Component
   ═══════════════════════════════════════════════════════════════════════════ */

.nav-header {
  position: relative;
  width: 100%;
  margin-bottom: 3rem;
  background: linear-gradient(180deg,
    rgba(10, 14, 15, 0.8) 0%,
    rgba(10, 10, 10, 0.95) 100%
  );
  backdrop-filter: blur(8px);
  border-bottom: 1px solid var(--void-border);
}

/* Compact header variant - single line, minimal padding */
.nav-header.compact {
  margin-bottom: 1.5rem;
}

.nav-header.compact .nav-inner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1.5rem;
  gap: 1rem;
}

.nav-header.compact .nav-current-location {
  gap: 0.5rem;
}

.nav-header.compact .nav-sigil {
  font-size: 1.5rem;
  filter: none;
}

.nav-header.compact .nav-location-name {
  font-size: var(--text-lg);
  text-shadow: none;
}

.nav-header.compact .nav-status {
  padding: 0.25rem 0.75rem;
  border-radius: 12px;
  font-size: var(--text-xs);
}

.nav-header.compact .nav-status-dot {
  width: 6px;
  height: 6px;
}

.nav-header.compact .nav-links {
  gap: 0.5rem;
}

.nav-header.compact .nav-link {
  padding: 0.4rem 0.6rem;
}

.nav-header.compact .nav-link-text {
  display: none;
}

/* Action button in header */
.nav-action-btn {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  padding: 0.4rem 0.8rem;
  background: rgba(0, 212, 170, 0.08);
  border: 1px solid rgba(0, 212, 170, 0.3);
  border-radius: 6px;
  color: var(--cyan);
  cursor: pointer;
  transition: all var(--transition-fast);
  white-space: nowrap;
}

.nav-action-btn:hover:not(:disabled) {
  background: rgba(0, 212, 170, 0.15);
  border-color: var(--cyan);
  box-shadow: 0 0 10px rgba(0, 212, 170, 0.2);
}

.nav-action-btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.nav-action-btn.loading {
  border-color: var(--gold);
  color: var(--gold);
  background: rgba(212, 175, 55, 0.08);
}

.nav-action-btn .action-icon {
  font-size: 1rem;
  line-height: 1;
}

.nav-action-btn.loading .action-icon {
  animation: spin 1s linear infinite;
}

/* Sacred geometry accent lines */
.nav-border-accent {
  height: 1px;
  background: linear-gradient(90deg,
    transparent 0%,
    var(--gold) 50%,
    transparent 100%
  );
  opacity: 0.3;
}

.nav-border-accent-bottom {
  height: 1px;
  background: repeating-linear-gradient(90deg,
    var(--moss) 0px,
    var(--moss) 2px,
    transparent 2px,
    transparent 8px,
    var(--cyan) 8px,
    var(--cyan) 10px,
    transparent 10px,
    transparent 20px
  );
  opacity: 0.15;
}

.nav-inner {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  gap: 2rem;
  padding: 1.5rem 3rem;
  max-width: 1600px;
  margin: 0 auto;
}

/* Current location (left side) */
.nav-current-location {
  display: flex;
  align-items: center;
  gap: 1rem;
  justify-self: start;
}

.nav-sigil {
  font-family: var(--font-serif);
  font-size: 2.5rem;
  color: var(--gold);
  line-height: 1;
  filter: drop-shadow(0 0 12px var(--gold-glow));
}

.nav-sigil.pulsing {
  animation: sacred-pulse 3s ease-in-out infinite;
}

@keyframes sacred-pulse {
  0%, 100% {
    opacity: 1;
    filter: drop-shadow(0 0 12px var(--gold-glow));
  }
  50% {
    opacity: 0.7;
    filter: drop-shadow(0 0 20px var(--gold-glow));
  }
}

.nav-location-name {
  font-family: var(--font-serif);
  font-size: var(--text-2xl);
  font-weight: 400;
  color: var(--gold);
  letter-spacing: 0.05em;
  margin: 0;
  text-shadow: 0 0 20px var(--gold-glow);
}

/* Status indicator (center) */
.nav-status {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.5rem 1.25rem;
  background: rgba(15, 15, 15, 0.5);
  border: 1px solid var(--moss);
  border-radius: 20px;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-secondary);
  justify-self: center;
  white-space: nowrap;
}

.nav-status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--moss-glow);
  box-shadow: 0 0 8px var(--moss-glow);
  animation: pulse 2s ease-in-out infinite;
}

.nav-status-text {
  color: var(--text-primary);
  font-size: var(--text-sm);
}

/* Navigation links (right side) */
.nav-links {
  display: flex;
  gap: 1.5rem;
  justify-self: end;
}

.nav-link {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.625rem 1.25rem;
  background: transparent;
  border: 1px solid var(--void-border);
  border-radius: 4px;
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  text-decoration: none;
  transition: all var(--transition-normal);
  cursor: pointer;
  position: relative;
  overflow: hidden;
}

.nav-link::before {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg,
    transparent,
    rgba(0, 212, 170, 0.1),
    transparent
  );
  transition: left var(--transition-slow);
}

.nav-link:hover::before {
  left: 100%;
}

.nav-link:hover {
  border-color: var(--cyan);
  color: var(--cyan);
  box-shadow: 0 0 20px var(--cyan-glow);
  transform: translateY(-1px);
}

.nav-link-sigil {
  font-family: var(--font-serif);
  font-size: 1.25rem;
  line-height: 1;
  opacity: 0.7;
  transition: all var(--transition-normal);
}

.nav-link:hover .nav-link-sigil {
  opacity: 1;
  filter: drop-shadow(0 0 8px var(--cyan));
}

.nav-link-text {
  font-weight: 400;
  letter-spacing: 0.02em;
}

/* Responsive behavior */
@media (max-width: 1024px) {
  .nav-inner {
    grid-template-columns: 1fr;
    grid-template-rows: auto auto auto;
    gap: 1.5rem;
    padding: 1.5rem 2rem;
  }

  .nav-current-location {
    justify-self: center;
  }

  .nav-status {
    justify-self: center;
  }

  .nav-links {
    justify-self: center;
    flex-wrap: wrap;
    justify-content: center;
  }

  .nav-location-name {
    font-size: var(--text-xl);
  }

  .nav-sigil {
    font-size: 2rem;
  }
}

@media (max-width: 640px) {
  .nav-inner {
    padding: 1rem 1.5rem;
    gap: 1rem;
  }

  .nav-location-name {
    font-size: var(--text-lg);
  }

  .nav-sigil {
    font-size: 1.75rem;
  }

  .nav-links {
    gap: 0.75rem;
    width: 100%;
  }

  .nav-link {
    flex: 1;
    justify-content: center;
    padding: 0.5rem 0.75rem;
    font-size: 0.8125rem;
  }

  .nav-link-text {
    display: none; /* Hide text on mobile, show only sigils */
  }

  .nav-link-sigil {
    font-size: 1.5rem;
  }

  .nav-status {
    font-size: 0.8125rem;
    padding: 0.4rem 1rem;
  }
}

/* Accessibility */
@media (prefers-reduced-motion: reduce) {
  .nav-sigil.pulsing,
  .nav-status-dot {
    animation: none;
  }

  .nav-link::before {
    display: none;
  }
}

/* ═══════════════════════════════════════════════════════════════════════════
   FIELD ACTIONS BAR - Quick actions below header
   ═══════════════════════════════════════════════════════════════════════════ */

.field-actions-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 3rem 1.5rem 3rem;
  max-width: 1600px;
  margin: -1.5rem auto 2rem auto;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: 0.625rem 1.5rem;
  background: transparent;
  border: 1px solid var(--moss);
  border-radius: 4px;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: all var(--transition-normal);
}

.action-btn:hover {
  border-color: var(--moss-glow);
  box-shadow: 0 0 20px rgba(124, 184, 124, 0.2);
  transform: translateY(-1px);
}

.action-icon {
  font-size: 1.125rem;
  line-height: 1;
  color: var(--moss-glow);
}

.action-text {
  font-weight: 400;
  letter-spacing: 0.02em;
}

@media (max-width: 1024px) {
  .field-actions-bar {
    padding: 0 2rem 1.5rem 2rem;
  }
}

@media (max-width: 640px) {
  .field-actions-bar {
    padding: 0 1.5rem 1rem 1.5rem;
    flex-direction: column;
    gap: 1rem;
    align-items: stretch;
  }

  .action-btn {
    justify-content: center;
  }
}
"#;
