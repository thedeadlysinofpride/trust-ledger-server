# TrustLedger - Milestones

## Milestone 1: Backend Foundation (Day 3)

Due: June 9, 2026

Deliverables:

- PostgreSQL database created and running
- All migrations generated and applied (groups, members, contributions, transactions)
- Basic CRUD endpoints returning data
- Server starts without errors on localhost:5150

Verification:

- curl <http://localhost:5150/api/health> returns 200
- Database has 5 tables (users, groups, members, contributions, transactions)
- Groups list endpoint returns empty array

---

## Milestone 2: Core Endpoints Complete (Day 8)

Due: June 14, 2026

Deliverables:

- Dashboard stats endpoint returns real aggregated data
- Groups CRUD working (list, get by id)
- Members list by group working
- Contribution recording endpoint working
- Trust score calculation engine functional

Verification:

- POST /api/auth/login returns JWT token
- GET /api/dashboard/stats returns totals with auth header
- GET /api/groups returns array of groups
- GET /api/groups/1/members returns members with trust scores
- POST /api/contributions creates record and updates trust score

---

## Milestone 3: Payout Feature Complete (Day 11)

Due: June 17, 2026

Deliverables:

- Payout endpoint triggers group disbursement
- Transactions recorded for each member
- Group totals reset after payout
- Next payout date advanced by one cycle

Verification:

- POST /api/groups/1/payout returns success
- New transaction records created for each member
- Group.total_contributed becomes 0
- Group.next_payout_date is 30 days from now

---

## Milestone 4: Moolre API Integration (Day 14)

Due: June 20, 2026

Deliverables:

- Moolre sandbox credentials obtained
- Collection flow works (member pays via Momo)
- Payout flow works (group sends money to members)
- Webhook handler processes Moolre callbacks

Verification:

- Contribution creation triggers Moolre collection request
- Moolre webhook updates contribution status from pending to completed
- Payout trigger sends Moolre payout requests for all members
- Failed transactions handled gracefully

---

## Milestone 5: USSD Endpoint Complete (Day 16)

Due: June 22, 2026

Deliverables:

- USSD state machine handles all menu flows
- Real data from database returned (not mock)
- Session management for multi-step interactions

Verification:

- POST /api/ussd with \*713# returns main menu
- Option 1 (Join Group) accepts group code
- Option 2 (Check Balance) returns member's savings
- Option 3 (Contribute) processes amount and confirms
- Option 4 (Trust Score) returns member's score

---

## Milestone 6: Frontend-Backend Integration (Day 18)

Due: June 24, 2026

Deliverables:

- Frontend API client points to real backend URL
- All dashboard pages show real data
- USSD simulator sends requests to backend
- Login works with seeded admin user

Verification:

- Frontend Settings page saves API URL
- Dashboard metrics match database values
- Groups page shows groups from database
- Clicking group shows members from database
- USSD simulator displays responses from backend
- Payout button triggers real backend endpoint

---

## Milestone 7: Deployment & Polish (Day 22)

Due: June 28, 2026

Deliverables:

- Backend deployed to production URL
- Frontend deployed to production URL
- CORS configured correctly
- Environment variables set on production
- Seed data loaded for demo

Verification:

- Production backend responds to /api/health
- Production frontend loads dashboard
- Frontend can login to production backend
- USSD simulator works on production
- Payout works on production (sandbox mode)

---

## Milestone 8: Demo Video & Submission (Day 25)

Due: July 1, 2026 (12 days before deadline)

Deliverables:

- 3-minute demo video recorded
- Video covers: login, dashboard, groups, members, trust scores, payout, USSD simulator
- Submission form filled
- Source code pushed to GitHub
- README updated with setup instructions

Verification:

- Video shows working product end-to-end
- No cuts or edits that hide broken features
- Video length under 4 minutes
- GitHub repository has both frontend and backend code

---

## Milestone 9: Buffer & Testing (Day 30-37)

Due: July 7 - July 13, 2026

Deliverables:

- All bugs found during testing fixed
- Edge cases handled (empty states, network errors)
- Performance checked (dashboard loads under 2 seconds)
- Final walkthrough with both team members
- Demo rehearsed

Verification:

- No console errors in production
- Mobile responsive layout works
- Dark/light theme works on all pages
- No green colors remaining in UI
- Trust scores calculate correctly for edge cases (new members, missed payments)

---

## Milestone Summary

| Milestone            | Due Date | Days from Start |
| -------------------- | -------- | --------------- |
| Backend Foundation   | June 9   | Day 3           |
| Core Endpoints       | June 14  | Day 8           |
| Payout Feature       | June 17  | Day 11          |
| Moolre Integration   | June 20  | Day 14          |
| USSD Endpoint        | June 22  | Day 16          |
| Frontend Integration | June 24  | Day 18          |
| Deployment           | June 28  | Day 22          |
| Demo Video           | July 1   | Day 25          |
| Buffer & Testing     | July 13  | Day 37          |

---

## Critical Path

The following tasks must be completed in order:

1. Database setup -> 2. Models creation -> 3. Groups endpoints -> 4. Members endpoints -> 5. Trust score calculation -> 6. Payout endpoint -> 7. Moolre integration -> 8. USSD endpoint -> 9. Frontend connection -> 10. Deployment

Do not skip steps. Do not work on USSD before Moolre. Do not work on frontend connection before backend endpoints are stable.

---

## Daily Check-In Questions

At the end of each day, answer these three questions:

1. What did we complete today?
2. What is blocked or not working?
3. What are we building tomorrow?

Keep answers in a shared document. Update milestones if behind schedule.

---

## Success Criteria for Hackathon Win

To win first place, the demo must show:

1. A judge can log into the dashboard
2. The dashboard shows real data (groups, members, contributions)
3. Clicking a group shows members with colored trust scores
4. The judge can click "Trigger Payout" and see success confirmation
5. The USSD simulator lets a judge type \*713# and navigate menus
6. The USSD simulator returns real data (not hardcoded mock responses)
7. The design is black, gray, and white only (no green or blue accents)
8. The entire demo works without refreshing or error messages

If all eight are true at the time of submission, first place is achievable.

---

## Risk Milestones

If any of these are not met by the specified date, escalate immediately:

| Risk Event                               | Deadline | Action                                      |
| ---------------------------------------- | -------- | ------------------------------------------- |
| Database not running                     | Day 1    | Switch to SQLite                            |
| Moolre sandbox credentials not received  | Day 9    | Contact Moolre support, build mock fallback |
| USSD simulator not connecting to backend | Day 16   | Hardcode demo responses as backup           |
| Deployment failing                       | Day 20   | Deploy to different provider                |
| Demo video not recorded                  | Day 25   | Record screen locally, edit later           |

---
