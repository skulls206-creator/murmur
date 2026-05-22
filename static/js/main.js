/**
 * Murmur Main JS
 *
 * Progressive enhancement for the Murmur Reddit frontend.
 * All core browsing works without JavaScript — this layer adds:
 *   - Toast notifications
 *   - Keyboard navigation helpers
 *   - Save/unsave actions
 *   - Smooth scrolling for anchor links
 */

(function() {
  'use strict';

  // ── Toast Notification System ──

  const toastContainer = document.querySelector('.toast-container');
  if (!toastContainer) {
    const container = document.createElement('div');
    container.className = 'toast-container';
    container.setAttribute('aria-live', 'polite');
    container.setAttribute('role', 'status');
    document.body.appendChild(container);
  }

  window.showToast = function(message, type = 'info') {
    const container = document.querySelector('.toast-container');
    const toast = document.createElement('div');
    toast.className = `toast toast--${type}`;
    toast.textContent = message;

    container.appendChild(toast);

    setTimeout(() => {
      toast.style.opacity = '0';
      toast.style.transform = 'translateY(8px)';
      toast.style.transition = 'opacity 200ms ease, transform 200ms ease';
      setTimeout(() => toast.remove(), 200);
    }, 3000);
  };

  // ── Save / Unsave Actions ──

  document.addEventListener('click', function(e) {
    const btn = e.target.closest('[data-action="save"], [data-action="unsave"]');
    if (!btn) return;

    e.preventDefault();
    const fullname = btn.dataset.fullname;
    const action = btn.dataset.action;

    fetch('/api/save', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ id: fullname, action: action }),
    })
      .then(r => r.json())
      .then(data => {
        if (data.success) {
          btn.dataset.action = action === 'save' ? 'unsave' : 'save';
          btn.textContent = action === 'save' ? 'Unsave' : 'Save';
          showToast(action === 'save' ? 'Saved!' : 'Unsaved', 'success');
        }
      })
      .catch(() => showToast('Failed to save', 'error'));
  });

  // ── Keyboard Shortcuts ──

  document.addEventListener('keydown', function(e) {
    // Don't trigger in input/textarea
    if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;

    // 's' — focus search
    if (e.key === 's' && !e.ctrlKey && !e.metaKey) {
      e.preventDefault();
      const searchInput = document.querySelector('.nav__search-input');
      if (searchInput) searchInput.focus();
    }

    // Escape — clear search focus
    if (e.key === 'Escape') {
      const active = document.activeElement;
      if (active && active.classList.contains('nav__search-input')) {
        active.blur();
      }
    }
  });

  // ── Smooth Scroll for Anchor Links ──

  document.addEventListener('click', function(e) {
    const link = e.target.closest('a[href^="#"]');
    if (!link) return;

    const target = document.querySelector(link.getAttribute('href'));
    if (target) {
      e.preventDefault();
      target.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  });

  console.log('Murmur UI loaded');
})();
