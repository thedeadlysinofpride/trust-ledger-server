# TrustLedger - Build Plan

## Project Overview

TrustLedger digitizes Ghana's informal economy by providing susu groups with digital record-keeping, trust scores, and automated payouts. Accessible via USSD (any phone) and web dashboard.

Hackathon Deadline: July 13, 2026 (Working MVP + demo video)

Team: 2 engineers

Stack:

- Frontend: React + TypeScript + Tailwind + TanStack Query (already built)
- Backend: Rust + Loco + PostgreSQL (to build)
- Payments: Moolre API

---

## Current Status (Day 1)

| Component          | Status        | Notes                                                             |
| ------------------ | ------------- | ----------------------------------------------------------------- |
| Frontend           | 90% complete  | Dashboard, groups, transactions, USSD simulator, dark/light theme |
| Backend            | Scaffold only | User auth exists, no business models yet                          |
| Database           | Schema only   | Users table exists, others need creation                          |
| Moolre Integration | Not started   | Need API keys                                                     |
| Deployment         | Not started   | Both frontend + backend                                           |

---

## 20-Day Build Timeline

### Phase 1: Backend Foundation (Days 1-3)

| Day | Task                                                           | Deliverable              |
| --- | -------------------------------------------------------------- | ------------------------ |
| 1   | Create database, run migrations                                | PostgreSQL running       |
| 2   | Generate models (groups, members, contributions, transactions) | All tables created       |
| 3   | Implement basic CRUD endpoints                                 | Groups list, get, create |

End of Phase 1: API returns mock data for groups and members

---

### Phase 2: Core Features (Days 4-8)

| Day | Task                            | Deliverable                                           |
| --- | ------------------------------- | ----------------------------------------------------- |
| 4   | Dashboard stats endpoint        | Total groups, members, contributions, avg trust score |
| 5   | Member management endpoints     | List members by group, add member                     |
| 6   | Contribution recording endpoint | POST /api/contributions                               |
| 7   | Trust score calculation engine  | Scores update after each contribution                 |
| 8   | Payout endpoint                 | POST /api/groups/:id/payout (mock Moolre first)       |

End of Phase 2: All MVP endpoints working with real database

---

### Phase 3: Moolre Integration (Days 9-11)

| Day | Task                                | Deliverable                               |
| --- | ----------------------------------- | ----------------------------------------- |
| 9   | Get Moolre sandbox credentials      | API keys from Moolre dashboard            |
| 10  | Implement collection (payment) flow | Member pays via Momo -> status updates    |
| 11  | Implement payout flow               | Group payout -> money sent to all members |

End of Phase 3: Real money moves in sandbox environment

---

### Phase 4: USSD Backend (Days 12-13)

| Day | Task                     | Deliverable                             |
| --- | ------------------------ | --------------------------------------- |
| 12  | USSD state machine       | Handles \*713#, menus, number inputs    |
| 13  | Connect USSD to database | Real group data, balances, trust scores |

End of Phase 4: USSD simulator in frontend talks to real backend

---

### Phase 5: Frontend-Backend Connection (Days 14-15)

| Day | Task                       | Deliverable                                    |
| --- | -------------------------- | ---------------------------------------------- |
| 14  | Update frontend API client | Point to real backend URL                      |
| 15  | Test all flows end-to-end  | Login -> Dashboard -> Groups -> Payout -> USSD |

End of Phase 5: Frontend and backend fully integrated

---

### Phase 6: Polish & Demo Prep (Days 16-20)

| Day | Task                       | Deliverable                                           |
| --- | -------------------------- | ----------------------------------------------------- |
| 16  | Error handling & logging   | API returns proper error responses                    |
| 17  | CORS & deployment config   | Backend deployable, frontend connects                 |
| 18  | Seed data for demo         | Groups, members, contributions with realistic numbers |
| 19  | Record demo video          | 3-minute walkthrough of all features                  |
| 20  | Final testing & submission | Everything working, video uploaded                    |

End of Phase 6: Ready for July 13 deadline

---

## Endpoints to Build (MVP)

### Authentication

| Method | Endpoint        | Status         |
| ------ | --------------- | -------------- |
| POST   | /api/auth/login | Already exists |

### Dashboard

| Method | Endpoint             | Status        |
| ------ | -------------------- | ------------- |
| GET    | /api/dashboard/stats | Need to build |

### Groups

| Method | Endpoint                | Status        |
| ------ | ----------------------- | ------------- |
| GET    | /api/groups             | Need to build |
| GET    | /api/groups/:id         | Need to build |
| POST   | /api/groups/:id/payout  | Need to build |
| GET    | /api/groups/:id/members | Need to build |
| POST   | /api/groups/:id/members | Need to build |

### Contributions

| Method | Endpoint                      | Status        |
| ------ | ----------------------------- | ------------- |
| POST   | /api/contributions            | Need to build |
| GET    | /api/groups/:id/contributions | Need to build |

### Transactions

| Method | Endpoint                     | Status        |
| ------ | ---------------------------- | ------------- |
| GET    | /api/transactions            | Need to build |
| GET    | /api/groups/:id/transactions | Need to build |

### USSD

| Method | Endpoint  | Status        |
| ------ | --------- | ------------- |
| POST   | /api/ussd | Need to build |

### Health

| Method | Endpoint    | Status        |
| ------ | ----------- | ------------- |
| GET    | /api/health | Need to build |

---

## Database Schema (MVP)

### users (already exists)

| Column   | Type   | Description |
| -------- | ------ | ----------- |
| id       | serial | Primary key |
| email    | text   | Admin login |
| password | text   | Hashed      |
| name     | text   | Admin name  |

### groups (to create)

| Column              | Type      | Description                  |
| ------------------- | --------- | ---------------------------- |
| id                  | serial    | Primary key                  |
| name                | text      | Group name                   |
| code                | text      | Unique short code (MKL, AOS) |
| description         | text      | Group description            |
| member_count        | int       | Number of active members     |
| total_contributed   | decimal   | Sum of all contributions     |
| contribution_amount | decimal   | Amount per cycle             |
| frequency           | text      | weekly or monthly            |
| next_payout_date    | timestamp | When cycle ends              |
| is_active           | bool      | Soft delete                  |

### members (to create)

| Column              | Type      | Description            |
| ------------------- | --------- | ---------------------- |
| id                  | serial    | Primary key            |
| name                | text      | Member name            |
| phone               | text      | Momo number            |
| trust_score         | int       | 0-100 calculated       |
| group_id            | int       | References groups      |
| total_contributed   | decimal   | Member's total savings |
| on_time_payments    | int       | Successful payments    |
| total_payments      | int       | Expected payments      |
| last_missed_payment | timestamp | Null if never missed   |
| status              | text      | active or inactive     |

### contributions (to create)

| Column     | Type      | Description                |
| ---------- | --------- | -------------------------- |
| id         | serial    | Primary key                |
| member_id  | int       | References members         |
| group_id   | int       | References groups          |
| amount     | decimal   | Amount paid                |
| status     | text      | pending, completed, failed |
| moolre_ref | text      | Transaction ID from Moolre |
| date       | timestamp | When contribution was made |

### transactions (to create)

| Column      | Type      | Description                   |
| ----------- | --------- | ----------------------------- |
| id          | serial    | Primary key                   |
| member_id   | int       | References members            |
| member_name | text      | Denormalized for fast display |
| group_id    | int       | References groups             |
| group_name  | text      | Denormalized for fast display |
| amount      | decimal   | Transaction amount            |
| type        | text      | contribution or payout        |
| status      | text      | pending, completed, failed    |
| moolre_ref  | text      | Transaction ID from Moolre    |
| date        | timestamp | When transaction occurred     |

---

## Trust Score Calculation

Formula:

(On-time payments / Total expected payments) \* 100 = Trust Score

On-time definition: Payment made within 24 hours of cycle deadline.

Score ranges:

| Range  | Label        | Display                           |
| ------ | ------------ | --------------------------------- |
| 90-100 | High Trust   | White text on white/10 background |
| 70-89  | Medium Trust | Light gray text on gray-600/30    |
| 50-69  | Low Trust    | Gray text on gray-700/30          |
| 0-49   | Poor Trust   | Dark gray text on gray-800/50     |

Recalculation triggers:

- After each successful contribution
- After missed payment detection (daily cron job)

---

## Moolre API Integration

### Sandbox Credentials

Required from Moolre dashboard:

- API Key
- API Secret
- Callback URL (for webhooks)

### Collection Flow (Member pays contribution)

1. Admin records contribution in dashboard
2. Backend calls Moolre POST /collections
3. Moolre sends payment request to member's Momo
4. Member approves on phone
5. Moolre calls our webhook with result
6. Backend updates contribution status
7. Trust score recalculated

### Payout Flow (Group disbursement)

1. Admin clicks "Trigger Payout" on dashboard
2. Backend calls Moolre POST /payouts for each member
3. Moolre sends money to each member's Momo
4. Moolre calls webhook for each transaction
5. Backend records transaction as completed
6. Group totals reset, next payout date advanced

---

## Deployment Plan

### Backend (Rust + Loco)

Option 1: Render.com (easiest)

- Connect GitHub repo
- Set build command: `cargo build --release`
- Set start command: `./target/release/trust-ledger-server`
- Add environment variables

Option 2: Railway.app

- Similar to Render, good for Rust

Option 3: DigitalOcean Droplet

- More control, more work

### Frontend (React + Vite)

Option 1: Vercel (easiest)

- Connect GitHub repo
- Build command: `npm run build`
- Output directory: `dist`

Option 2: Netlify

- Same as Vercel

Option 3: Render Static Sites

- Works well

### Environment Variables

Backend (.env):

```.env
APP_ENV=production
APP_SECRET=<generate-random>
DATABASE_URL=<production-db-url>
JWT_SECRET=<generate-random>
MOOLRE_API_KEY=<from-moolre>
MOOLRE_API_URL=https://api.moolre.com/v1
FRONTEND_URL=<frontend-url>
```

Frontend (.env):

```.env
VITE_API_BASE_URL=<backend-url>/api
VITE_USE_MOCK=false
```

---

## Risk Management

| Risk                         | Probability | Mitigation                                     |
| ---------------------------- | ----------- | ---------------------------------------------- |
| Moolre API sandbox issues    | Medium      | Have mock fallback, test early                 |
| Database connection problems | Low         | Use SQLite as backup                           |
| CORS blocking frontend       | Low         | Configure properly on day 1                    |
| Rust compilation errors      | Medium      | Build frequently, don't wait                   |
| USSD carrier integration     | High        | Use simulator for demo, defer real integration |
| Team member unavailable      | Low         | Clear daily check-ins, shared docs             |

---

## Success Checklist for July 13

- [ ] Backend deploys and responds to /api/health
- [ ] Frontend connects to backend via Settings page
- [ ] Login works with seeded admin user
- [ ] Dashboard shows real stats from database
- [ ] Groups page shows seeded groups
- [ ] Group detail page shows members with trust scores
- [ ] Payout button triggers Moolre API (sandbox)
- [ ] USSD simulator talks to backend, not mock
- [ ] Trust scores update after contributions
- [ ] Dark/light theme works on all pages
- [ ] No green colors anywhere (pure grayscale)
- [ ] Demo video recorded (3 minutes max)
- [ ] Source code on GitHub (private or public)
- [ ] Submission link sent before deadline

---

## Daily Workflow

Start each day with:

```bash
cd trust-ledger-server
git pull
cargo run
```

End each day with:

```bash
git add .
git commit -m "description of what was built"
git push
```

Keep a running log of what works and what doesn't. Test the frontend against the backend at least once per day.
