# Murmur 🥷

> Browse Reddit privately. No tracking. No ads. No data collection.

**Murmur** is a modern, privacy-first Reddit frontend proxy. Think Invidious for Reddit — all traffic is routed through your server so users are never exposed to Reddit's tracking, ads, or data collection. Written in Rust with a beautiful dark-mode-first UI.

---

## Architecture

```
┌─────────────┐     ┌──────────────────────────────────────┐     ┌──────────────┐
│             │     │              Murmur Server            │     │              │
│   Browser   │────▶│                                      │────▶│  Reddit API  │
│  (User)     │     │  ┌──────────┐  ┌──────────────────┐  │     │  (Upstream)  │
│             │     │  │  Axum    │  │  Template Engine  │  │     │              │
│  No JS req  │     │  │  Router  │──│  (Askama/Tera)    │  │     │  oauth.      │
│  for core   │     │  │          │  │                   │  │     │  reddit.com  │
│             │     │  │  /r/sub  │  │  HTML + CSS       │  │     │              │
│  Progressive│     │  │  /search │  │  Design System    │  │     │  www.reddit  │
│  JS for UX  │     │  │  /u/user │  │                   │  │     │  .com/api    │
│             │     │  │  /media  │  └──────────────────┘  │     │              │
│  CSP:       │     │  └──────────┘                        │     │  oauth.      │
│  No 3rd     │     │       │                              │     │  reddit.com  │
│  party      │     │  ┌────┴─────┐     ┌───────────────┐  │     │  /api/v1     │
│  requests   │     │  │  Cache    │     │  Media Proxy  │  │     │              │
│             │     │  │  (moka)   │     │  (base64 URL) │  │     │  i.redd.it   │
└─────────────┘     │  │  Redis opt│     │  ┌──────────┐ │  │     │  v.redd.it   │
                    │  └──────────┘     │  │ Fetch +   │ │  │     │  preview.    │
                    │                   │  │ Resize    │ │  │     │  redd.it     │
                    │                   │  │ Stream    │ │  │     │              │
                    │                   │  └──────────┘ │  │     └──────────────┘
                    │                   └───────────────┘  │
                    └──────────────────────────────────────┘

### Request Flow

1. **User's browser** requests `murmur.example.com/r/programming`
2. **Axum router** receives the request, applies middleware (CSP headers, auth check, cache lookup)
3. **RedditClient** fetches data from Reddit's OAuth API with Murmur's own credentials — NO user cookies or IP forwarded
4. **Cache layer** stores the result for TTL seconds (in-memory moka or optional Redis)
5. **Media proxy** rewrites all `i.redd.it`, `v.redd.it`, `preview.redd.it` URLs to Murmur's `/media/<base64>` endpoint
6. **Template engine** renders the HTML with the design system CSS
7. **Response** is sent with strict CSP headers — browser cannot make any external requests

### Privacy Guarantees

| Feature | Murmur | Redlib | Reddit |
|---------|--------|--------|--------|
| Media proxied server-side | ✅ | ✅ | ❌ |
| No JavaScript required | ✅ | ✅ | ❌ |
| No tracking pixels | ✅ | ✅ | ❌ |
| HTTP-only cookies | ✅ | ✅ | ❌ |
| Strict CSP | ✅ | ❌ | ❌ |
| Tor/I2P support | ✅ | ✅ | ❌ |
| Modern responsive UI | ✅ | ❌ | ❌ |
| Dark/Light/System themes | ✅ | ❌ | ❌ |
| WCAG 2.1 AA | ✅ | ❌ | ❌ |

---

## Features

### Core Reddit Features
- ✅ Browse frontpage, subreddits, posts, comments
- ✅ User profiles and post history
- ✅ Search (posts, subreddits, users)
- ✅ Subreddit discovery (autocomplete)
- ✅ Full OAuth login — vote, comment, post, save
- ✅ Post text, links, images, videos
- ✅ Comment threads with nested replies
- ✅ Vote (upvote/downvote/unvote)
- ✅ User inbox, saved posts, moderation tools
- ✅ NSFW with blur overlay and instance toggle
- ✅ Flair (post and user)
- ✅ Multireddit support

### UI Features
- 🌗 Dark mode default, light mode, system preference
- 📐 Three feed views: Card, Compact, Classic
- 🎨 Design system with CSS custom properties
- 🪟 Frosted glass nav bar
- 🌊 Infinite scroll (progressive enhancement)
- ✔️ WCAG 2.1 AA accessible
- 📱 Mobile-first responsive
- 🚫 No JavaScript required for core browsing
- ⚡ Skeleton loading states

### Privacy & Security
- 🔒 Full media proxy — no direct CDN connections
- 🛡️ Strict CSP: no inline scripts (unless nonced), no external connections
- 🍪 No tracking cookies, no fingerprinting
- 🔐 Encrypted session cookies
- 🌐 Tor-friendly (SOCKS5 proxy support)
- ⚙️ Configurable instance settings

---

## Quick Start

### Using Docker

```bash
# Clone and configure
git clone https://github.com/skulls206-creator/murmur
cd murmur
cp .env.example .env
# Edit .env with your Reddit API credentials

docker compose up -d
```

### From Source

```bash
# Prerequisites: Rust 1.79+, OpenSSL dev headers
cargo build --release
cp .env.example .env
# Edit .env with your credentials
./target/release/murmur
```

### Configuration

See `.env.example` for all options. Key settings:

| Variable | Default | Description |
|----------|---------|-------------|
| `MURMUR_BIND_ADDR` | `0.0.0.0:8080` | Server listen address |
| `MURMUR_BASE_URL` | `http://localhost:8080` | Public URL (for OAuth) |
| `MURMUR_REDDIT_CLIENT_ID` | — | Reddit API client ID |
| `MURMUR_REDDIT_CLIENT_SECRET` | — | Reddit API secret |
| `MURMUR_COOKIE_SECRET` | — | 32-byte hex for cookie encryption |
| `MURMUR_PROXY_MEDIA` | `true` | Proxy media server-side |
| `MURMUR_ALLOW_NSFW` | `true` | Allow NSFW content |
| `MURMUR_REQUIRE_LOGIN` | `false` | Require auth to browse |
| `MURMUR_SOCKS5_PROXY` | — | SOCKS5 for Tor/I2P |

### Getting Reddit API Credentials

1. Go to https://www.reddit.com/prefs/apps
2. Click "Create App" or "Create Another App"
3. Select **web app**
4. Set redirect URI to `http://localhost:8080/auth/callback` (or your deployed URL)
5. Copy the client ID (under the app name) and secret

---

## Project Structure

```
murmur/
├── Cargo.toml              # Rust dependencies
├── Dockerfile               # Multi-stage build
├── .env.example             # Environment config template
├── src/
│   ├── main.rs              # Entry point
│   ├── config.rs            # Environment-based config
│   ├── error.rs             # Unified error handling
│   ├── router.rs            # Axum router + middleware setup
│   ├── templates.rs         # Template filters (time, media URL)
│   ├── models/              # Data models
│   │   ├── mod.rs           # Sort mode, time filter, feed view
│   │   ├── post.rs          # Post, PostMedia, GalleryItem
│   │   ├── comment.rs       # Comment tree structure
│   │   ├── subreddit.rs     # Subreddit, rules, moderators
│   │   └── user.rs          # RedditUser, UserSession, Message
│   ├── proxy/               # Reddit API client
│   │   ├── mod.rs
│   │   ├── reddit_api.rs    # All Reddit API calls
│   │   └── media_proxy.rs   # Media fetching, caching, encoding
│   ├── middleware/           # Request middleware
│   │   ├── mod.rs
│   │   ├── csp.rs           # Content Security Policy + security headers
│   │   ├── auth.rs          # Session cookie auth
│   │   └── cache.rs         # Response caching layer
│   └── routes/              # Route handlers
│       ├── mod.rs
│       ├── frontpage.rs     # / and /feed
│       ├── subreddit.rs     # /r/{name}
│       ├── post.rs          # /r/{name}/comments/{id}
│       ├── search.rs        # /search
│       ├── auth.rs          # /auth/* (OAuth flow)
│       ├── user.rs          # /u/{username}
│       ├── api.rs           # /api/* (vote, comment, health)
│       ├── settings.rs      # /settings
│       └── subreddit_discovery.rs  # /discover
├── templates/               # Askama HTML templates
│   ├── base.html            # Layout with nav, theme support
│   ├── components/          # Reusable components
│   │   ├── post_card.html   # Post card (3 views: card/compact/classic)
│   │   ├── comment.html     # Recursive comment thread
│   │   └── subreddit_sidebar.html  # Sidebar with rules/mods
│   ├── pages/               # Page layouts
│   │   ├── frontpage.html
│   │   ├── subreddit.html
│   │   ├── post.html
│   │   ├── search.html
│   │   ├── login.html
│   │   └── settings.html
│   └── partials/            # Partial templates
│       ├── feed_items.html
│       └── comment_thread.html
└── static/                  # Static assets
    ├── css/
    │   └── design-system.css  # Complete design system (45KB)
    ├── js/
    │   ├── main.js           # Core progressive enhancement
    │   ├── theme.js          # Dark/light/system theme
    │   ├── voting.js         # Optimistic vote UI
    │   ├── comments.js       # Comment reply forms
    │   └── infinite-scroll.js # Feed pagination
    ├── robots.txt
    └── manifest.json [planned]
```

---

## Design System

Murmur's visual identity is built around:

### Colors
- **Primary**: Deep violet (#7C5CFC) — trust, privacy
- **Accent**: Coral (#FF6B6B) — votes, energy
- **Upvote/Downvote**: Coral / Periwinkle blue
- **Background**: Near-black (#0C0C12) with subtle violet undertones
- **Surfaces**: Layered cards with 3 levels of depth

### Typography
- **Font**: Inter (system sans-serif fallback)
- **Scale**: Modular scale 1.25 (12px → 36px)
- **Weights**: 400 (normal), 500 (medium), 600 (semibold), 700 (bold)

### Theming
- Dark mode is default — not an afterthought
- Light mode is equally polished
- System preference detection (`prefers-color-scheme`)
- Theme persisted in localStorage
- No FOUC (theme applied before render via blocking script)

### Accessibility
- WCAG 2.1 AA contrast ratios
- `prefers-reduced-motion` respects user settings
- Proper ARIA labels on all interactive elements
- Focus-visible indicators
- Keyboard navigation (s key for search, escape to blur)
- Color not the only differentiator

---

## Why Not Redlib?

Redlib is great engineering — but its UI is stuck in 2015. Murmur differentiates on:

1. **Visual design** — modern, spacious, dark-mode-first with a proper design system
2. **CSS custom properties** — themable, maintainable, lightweight
3. **Progressive enhancement** — core works without JS, but JS adds polish
4. **Three feed views** — Card, Compact, Classic (Redlib has one)
5. **Proper typography** — Inter font, modular scale, comfortable reading
6. **WCAG compliance** — Redlib lacks proper ARIA and keyboard support
7. **Rust backend** — same language, but cleaner architecture with Axum + Askama
8. **Developer experience** — clean module structure, comprehensive error handling

---

## License

AGPL-3.0 — See `LICENSE`.

---

*Built by skullsdev. Privacy is not optional.* 🥷
