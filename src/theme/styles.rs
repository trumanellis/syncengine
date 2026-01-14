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
  max-width: 1000px;
  margin: 0 auto;
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

.empty-task-state {
  color: var(--text-muted);
  font-style: italic;
  font-size: var(--text-sm);
  padding: 1rem;
  text-align: center;
  border: 1px dashed var(--void-border);
  border-radius: 4px;
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
"#;
