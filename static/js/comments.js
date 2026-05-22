/**
 * Murmur Comments (Progressive Enhancement)
 *
 * Handles comment submission and reply forms.
 * Also manages the "load more comments" interaction.
 */

(function() {
  'use strict';

  // ── Comment Form Submission ──

  document.addEventListener('submit', function(e) {
    const form = e.target.closest('.comment-form');
    if (!form) return;

    e.preventDefault();

    const textarea = form.querySelector('textarea');
    const text = textarea.value.trim();
    if (!text) return;

    const postFullname = form.dataset.post;
    const submitBtn = form.querySelector('button[type="submit"]');
    submitBtn.disabled = true;
    submitBtn.textContent = 'Posting...';

    // Determine parent — either the post or a specific comment
    const parent = form.dataset.parent || postFullname;

    fetch('/api/comment', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ parent: parent, text: text }),
    })
      .then(r => r.json())
      .then(data => {
        if (data.success && data.comment) {
          textarea.value = '';
          showToast('Comment posted!', 'success');

          // Optionally prepend the comment to the thread
          // For now, just refresh (simplest approach)
          if (window.location.hash) {
            window.location.reload();
          }
        } else {
          showToast('Failed to post comment', 'error');
        }
      })
      .catch(() => {
        showToast('Failed to post comment — network error', 'error');
      })
      .finally(() => {
        submitBtn.disabled = false;
        submitBtn.textContent = 'Comment';
      });
  });

  // ── Reply Forms (expandable inline) ──

  document.addEventListener('click', function(e) {
    const btn = e.target.closest('[data-action="reply"]');
    if (!btn) return;

    e.preventDefault();

    const fullname = btn.dataset.fullname;
    const commentEl = btn.closest('.comment');

    // Check if reply form already exists
    if (commentEl.querySelector('.comment-form')) return;

    const form = document.createElement('form');
    form.className = 'comment-form';
    form.dataset.parent = fullname;
    form.style.marginTop = 'var(--space-3)';

    form.innerHTML = `
      <textarea
        class="form-textarea"
        name="text"
        placeholder="Write a reply..."
        aria-label="Write a reply"
        rows="3"
        style="margin-bottom: var(--space-2);"
      ></textarea>
      <div style="display: flex; gap: var(--space-2);">
        <button type="submit" class="btn btn--primary btn--sm">Reply</button>
        <button type="button" class="btn btn--ghost btn--sm cancel-reply">Cancel</button>
      </div>
    `;

    const actions = commentEl.querySelector('.comment__actions');
    actions.parentNode.insertBefore(form, actions.nextSibling);

    // Focus the textarea
    const textarea = form.querySelector('textarea');
    textarea.focus();

    // Cancel button
    form.querySelector('.cancel-reply').addEventListener('click', () => {
      form.remove();
    });
  });
})();
