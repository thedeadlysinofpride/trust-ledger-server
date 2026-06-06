# TrustLedger - About

## What is TrustLedger?

TrustLedger is a financial operating system for Ghana's informal economy. It digitizes susu groups, market trader savings, and rotational savings systems (ROSCAs) by providing:

1. **Digital record-keeping** that replaces paper notebooks
2. **Trust scores** that create financial identity for the unbanked
3. **Automated payouts** that remove collection risk
4. **USSD access** for users without smartphones

## The Problem We're Solving

### In Ghana Today

- **80% of the workforce** operates in the informal economy
- **Susu collectors** carry notebooks with thousands of cedis in cash
- **Notebooks get lost** — savings disappear overnight
- **No credit history** means no loans, no formal financial services
- **Momo payments exist** but no system ties them to group savings

### Real Example

> A tomato trader at Makola Market saves GHS 50 weekly with her susu group of 20 women. After 6 months, the collector loses the notebook. GHS 24,000 in savings — gone. No record. No recourse.

TrustLedger prevents this.

## What TrustLedger Does

### For Susu Group Members (via USSD)

| Action           | How it works                      |
| ---------------- | --------------------------------- |
| Join group       | Dial \*713# → enter group code    |
| Pay contribution | USSD menu → amount → Momo payment |
| Check balance    | Dial \*713# → "My Balance"        |
| View trust score | Dial \*713# → "My Trust Score"    |

No smartphone required. Works on any phone.

### For Group Admins (via Web Dashboard)

| Feature             | What they see                                        |
| ------------------- | ---------------------------------------------------- |
| Group overview      | Members, total saved, next payout date               |
| Member trust scores | Green/yellow/red indicators with explanations        |
| Transaction history | Every contribution recorded permanently              |
| One-click payouts   | Automatic disbursement to all members via Moolre API |

### For Judges (During Demo)

The dashboard includes a **live USSD simulator** — a terminal-style box where judges can type `*713#` and see real responses. No need to pick up a phone.

## The Core Innovation: Trust Scores

Every member gets a trust score calculated from:

(On-time payments / Expected payments) × 100

- **90-100%** → White badge, "High Trust"
- **70-89%** → Light gray badge, "Medium Trust"
- **50-69%** → Dark gray badge, "Low Trust"
- **Below 50%** → Darkest gray badge, "Poor Trust"

This trust score **follows the member across groups** and becomes their first financial identity. Banks don't serve these people — but TrustLedger does.

## Technical Architecture

### Frontend (React + TypeScript)

- Admin dashboard with dark/light theme
- Live USSD simulator for demos
- Group management, member tracking, transaction history
- Connects to backend via REST API

### Backend (Rust + Loco)

- JWT authentication for admin access
- PostgreSQL database (SQLite for MVP)
- Trust score calculation engine
- Moolre API integration for Momo payments

### API Endpoints (MVP)

| Endpoint                      | Purpose                   |
| ----------------------------- | ------------------------- |
| `POST /api/auth/login`        | Admin authentication      |
| `GET /api/dashboard/stats`    | Key metrics for dashboard |
| `GET /api/groups`             | List all susu groups      |
| `GET /api/groups/:id/members` | Members with trust scores |
| `POST /api/groups/:id/payout` | Trigger automatic payout  |
| `POST /api/ussd`              | USSD menu handler         |
| `GET /api/health`             | Service health check      |

## What Makes This a Winning Hackathon Entry

### 1. It's Built for Ghana

- USSD-first (not smartphone-first)
- Momo payments (not cards or bank transfers)
- Susu groups (not individual savings)

### 2. The Trust Score Is Novel

Every other team will build "digitize susu payments." Only we're building **financial identity**.

### 3. It's Demo-Ready

Judges can:

- Open the dashboard (sees real data)
- Click the USSD simulator (types \*713# and navigates)
- Trigger a payout (watches money move)

### 4. The Stack Shows Technical Depth

| Choice             | Why It Impresses Judges            |
| ------------------ | ---------------------------------- |
| Rust + Loco        | Performance, memory safety, modern |
| React + TypeScript | Production-grade frontend          |
| CSS tokens         | Single source of truth for design  |
| Moolre API         | Real payment integration, not mock |

## What We're NOT Building (Yet)

| Feature                    | Why Cut                    |
| -------------------------- | -------------------------- |
| Mobile app                 | USSD covers feature phones |
| Individual income tracking | Too complex for MVP        |
| Loans                      | Requires underwriting      |
| Multiple payment methods   | Momo is enough for Ghana   |
| Real-time chat             | Not core to the problem    |

## The Goal by July 13

A working MVP that demonstrates:

1. **End-to-end susu group management** (create group → join → contribute → payout)
2. **Trust scores** that update based on payment history
3. **USSD access** via simulator (ready to deploy to real carrier)
4. **Admin dashboard** that shows everything in real time

## The Pitch Line

> "TrustLedger: Financial identity for the 80% of Ghanaians that banks ignore. We digitize susu groups, build trust scores through payment history, and enable automatic payouts — all accessible from any phone via USSD."

## Why This Matters

This isn't a "nice to have." This is infrastructure.

When a market trader has a trust score, she can:

- Get a loan to buy inventory
- Prove income for a child's school fees
- Build a financial life outside the cash economy

That's the real win.
