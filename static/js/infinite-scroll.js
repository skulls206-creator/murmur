/**
 * Murmur Infinite Scroll
 *
 * Loads more posts when the user scrolls near the bottom of the feed.
 * Uses IntersectionObserver for performance.
 * Degrades gracefully — no JS, no infinite scroll, normal pagination still works.
 */

(function() {
  'use strict';

  const trigger = document.querySelector('.infinite-scroll-trigger');
  if (!trigger) return;

  let loading = false;
  let hasMore = true;

  const observer = new IntersectionObserver(async (entries) => {
    const entry = entries[0];
    if (!entry.isIntersecting || loading || !hasMore) return;

    loading = true;

    // Show loading indicator
    const loadingEl = document.createElement('div');
    loadingEl.className = 'infinite-scroll-loading';
    loadingEl.innerHTML = '<div class="spinner"></div>';
    trigger.parentNode.insertBefore(loadingEl, trigger.nextSibling);

    try {
      const after = trigger.dataset.after;
      const sort = trigger.dataset.sort;
      const view = trigger.dataset.view;
      const subreddit = trigger.dataset.subreddit;
      const query = trigger.dataset.query;

      // Determine the URL
      let url;
      if (subreddit) {
        url = `/r/${subreddit}?sort=${sort}&view=${view}&after=${after}&count=${document.querySelectorAll('.post-card').length}`;
      } else if (query) {
        url = `/search?q=${query}&type=${trigger.dataset.type || 'posts'}&after=${after}`;
      } else {
        url = `/?sort=${sort}&view=${view}&after=${after}&count=${document.querySelectorAll('.post-card').length}`;
      }

      // HTMX-style: fetch just the feed items
      url = '/feed' + url.substring(url.indexOf('?'));

      const resp = await fetch(url);
      const html = await resp.text();

      // Parse the HTML and extract post cards
      const temp = document.createElement('div');
      temp.innerHTML = html;

      const feed = document.getElementById('feed');
      const newPosts = temp.querySelectorAll('.post-card');

      if (newPosts.length === 0) {
        hasMore = false;
        loadingEl.remove();
        return;
      }

      newPosts.forEach(post => {
        // Add fade-in animation
        post.classList.add('fade-in');
        feed.appendChild(post);
      });

      // Update trigger
      const lastPost = newPosts[newPosts.length - 1];
      const newAfter = lastPost ? lastPost.closest('[data-fullname]')?.dataset.fullname : null;
      if (newAfter) {
        trigger.dataset.after = newAfter;
      } else {
        hasMore = false;
      }
    } catch (err) {
      console.error('Infinite scroll failed:', err);
      hasMore = false;
    } finally {
      loadingEl.remove();
      loading = false;
    }
  }, {
    rootMargin: '400px', // Start loading before user reaches the trigger
  });

  observer.observe(trigger);
})();
