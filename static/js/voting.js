/**
 * Murmur Voting (Progressive Enhancement)
 *
 * Handles upvote/downvote on posts and comments via the JSON API.
 * Falls back gracefully if JS is unavailable — links still work.
 */

(function() {
  'use strict';

  document.addEventListener('click', function(e) {
    const btn = e.target.closest('.vote-btn');
    if (!btn) return;

    e.preventDefault();

    const fullname = btn.dataset.fullname;
    const isUp = btn.dataset.vote === 'up';
    const isActive = btn.classList.contains('active');

    // Determine direction: toggle if active, otherwise set
    let dir;
    if (isActive) {
      dir = 0; // unvote
    } else {
      dir = isUp ? 1 : -1;
    }

    // Optimistic UI update
    const card = btn.closest('[data-fullname]');
    const scoreEl = card ? card.querySelector('.vote-score') : null;
    const currentScore = scoreEl ? parseInt(scoreEl.dataset.score || '0', 10) : 0;
    const upBtn = card ? card.querySelector('.vote-btn--up') : null;
    const downBtn = card ? card.querySelector('.vote-btn--down') : null;

    // Optimistic update
    let newScore = currentScore;
    if (dir === 1) {
      // Upvoting
      if (downBtn && downBtn.classList.contains('active')) {
        newScore += 2; // was downvoted, now upvoted
        downBtn.classList.remove('active');
      } else if (!upBtn.classList.contains('active')) {
        newScore += 1;
      }
      upBtn.classList.add('active');
      if (scoreEl) {
        scoreEl.classList.remove('vote-score--downvoted');
        scoreEl.classList.add('vote-score--upvoted');
      }
    } else if (dir === -1) {
      // Downvoting
      if (upBtn && upBtn.classList.contains('active')) {
        newScore -= 2; // was upvoted, now downvoted
        upBtn.classList.remove('active');
      } else if (!downBtn.classList.contains('active')) {
        newScore -= 1;
      }
      downBtn.classList.add('active');
      if (scoreEl) {
        scoreEl.classList.remove('vote-score--upvoted');
        scoreEl.classList.add('vote-score--downvoted');
      }
    } else {
      // Unvote
      if (upBtn && upBtn.classList.contains('active')) {
        newScore -= 1;
        upBtn.classList.remove('active');
      }
      if (downBtn && downBtn.classList.contains('active')) {
        newScore += 1;
        downBtn.classList.remove('active');
      }
      if (scoreEl) {
        scoreEl.classList.remove('vote-score--upvoted', 'vote-score--downvoted');
      }
    }

    if (scoreEl) {
      scoreEl.textContent = newScore;
      scoreEl.dataset.score = newScore;
    }

    // Send to API
    fetch('/api/vote', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ id: fullname, dir: dir }),
    })
      .then(r => r.json())
      .then(data => {
        if (!data.success) {
          // Revert on failure
          if (scoreEl) {
            scoreEl.textContent = currentScore;
            scoreEl.dataset.score = currentScore;
          }
          showToast('Vote failed', 'error');
        }
      })
      .catch(() => {
        // Revert on network error
        if (scoreEl) {
          scoreEl.textContent = currentScore;
          scoreEl.dataset.score = currentScore;
        }
        showToast('Vote failed — network error', 'error');
      });
  });
})();
