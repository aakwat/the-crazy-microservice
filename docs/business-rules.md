# Support Tickets: Business Rules

Status: draft for review, 2026-11-13. Target: **one local company running its own customer support
in-house** (self-hosted, one organisation, a small support team). Business policy only: no code, UI or
technology choices.

## 0. How to read this

- Each rule is **Mandatory** (always true) or **Configurable** (an Admin setting with a stated default),
  and **MVP** or **Later**.
- **D-nn** marks an open business decision; the rule states a proposed default and section 15 lists it.
- Times are in the company's time zone. **Business time** counts only business hours and skips holidays.

| Term          | Meaning                                                                          |
| ------------- | -------------------------------------------------------------------------------- |
| Company       | The one organisation that owns and runs the system.                              |
| Staff         | Admins and Agents of the company.                                                |
| Customer      | A person outside the company who asks for support.                               |
| Team          | An internal group of agents (e.g. Billing, Technical). Never shown to customers. |
| Category      | A customer-facing topic ("Problem with my invoice") linked to one team.          |
| Assignee      | The one agent responsible for a ticket. Never shown to customers.                |
| Public reply  | A message the customer can read.                                                 |
| Internal note | A message only staff can read.                                                   |

## 1. Principles

These decide every conflict. A rule that breaks one is wrong.

| ID | Principle                                                                                                                        |
| -- | -------------------------------------------------------------------------------------------------------------------------------- |
| P1 | The customer deals with **the company**, not with people or teams: no staff names, no team names, no routing.                    |
| P2 | The customer does five things: submit a problem, keep a reference, talk through the ticket, follow progress, confirm the result. |
| P3 | Every open ticket always has someone responsible: an agent, or a team queue the Admins watch.                                    |
| P4 | Promises are measurable: each response and resolution target has one clock and one breach moment.                                |
| P5 | History is evidence: nothing that happened on a ticket is edited or deleted, only added to.                                      |
| P6 | Keep it small: a feature that a 5-person support team would not use does not belong in the MVP.                                  |

## 2. Company setup (SET)

### SET-001: One company per installation

`Mandatory · MVP`

- **Description:** The system serves exactly one company. All customers, staff, tickets and settings
  belong to it.
- **Purpose:** In-house use; no tenant management or separation logic to run.
- **Trigger / conditions:** Installation.
- **Behavior:** The first Admin account is created at installation; everything else is created by Admins.
- **Restrictions:** A second company means a second installation.
- **Edge cases:** None in scope.
- **Acceptance:** There is no way to create a second company inside one installation.

### SET-002: Configuration

`Mandatory · MVP`

- **Description:** Admins configure: company name, support email address, time zone, business hours,
  holidays, categories, teams, category-to-team links, SLA targets, reopen window, auto-close period,
  and how replies are signed (COMM-003).
- **Behavior:** Changes apply from the moment they are saved; open tickets keep the SLA targets they
  started with (SLA-007). Every change is audited.
- **Restrictions:** A category or team with open tickets cannot be deleted; it can be retired (hidden
  from new tickets).
- **Edge cases:** A time-zone change: running clocks continue in the new zone.
- **Acceptance:** A settings change never changes a past timestamp, SLA result or ticket history.

## 3. Roles (ROLE)

### ROLE-001: Roles

`Mandatory · MVP`

| Role     | Summary                                                                                                         |
| -------- | --------------------------------------------------------------------------------------------------------------- |
| Admin    | Configures the system, manages staff and roles, sees and acts on every ticket, handles escalations and reports. |
| Agent    | Works tickets of their teams: assigned ones and the team queue.                                                 |
| Customer | Submits and follows their own tickets.                                                                          |

- **Purpose:** Three roles a small team understands; supervisor duties belong to Admins.
- **Restrictions:** A person is either staff or customer, never both. At least one active Admin always exists.
- **Acceptance:** Removing or demoting the last active Admin is refused.

### ROLE-002: Staff accounts

`Mandatory · MVP`

- **Description:** Admins create staff accounts (email, role, teams). A staff account is active after
  the person verifies their email.
- **Behavior:** Deactivation is immediate: the person loses access and their open tickets are reassigned
  (ASSIGN-005). Deactivated staff stay as authors in history (P5).
- **Restrictions:** Only Admins grant roles.
- **Edge cases:** Creating staff with an email that belongs to a customer: refused.
- **Acceptance:** After deactivation the person can do nothing and owns no open ticket.

### ROLE-003: Admin permissions

`Mandatory · MVP`

- **May:** everything in SET-002; manage staff and roles; see, assign, reassign and act on every ticket;
  change any priority; close as spam or duplicate; excuse SLA breaches; block customers; redact
  messages; read all reports and the audit log.
- **Acceptance:** Every action in the permission matrix (14.2) marked for Admin is allowed for Admin.

### ROLE-004: Agent permissions

`Mandatory · MVP`

- **May:** see tickets assigned to them and the queues of their teams; take a ticket from their team
  queue; reply; write internal notes; change status within the lifecycle (section 13); raise priority;
  lower it to Medium or Low; mark a probable duplicate for an Admin to confirm.
- **May not:** see tickets of other teams, reassign a colleague's ticket, change settings, see other
  agents' numbers.
- **Acceptance:** An Agent asking for a ticket outside their teams gets "not found".

### ROLE-005: Customer permissions

`Mandatory · MVP`

- **May:** submit tickets, see their own tickets, reply, attach files, withdraw, confirm or reopen a
  resolution within the window, rate the service.
- **May not:** see other customers' tickets, internal notes, staff names, teams, assignment, priority or
  SLA timers.
- **Acceptance:** No customer-visible screen, email or response contains a staff name, team, internal
  note, assignment or another customer's data.

### ROLE-006: Role changes apply at once

`Mandatory · MVP`

- **Description:** A role or team change applies to the person's next action, not only after they sign
  in again.
- **Acceptance:** After a demotion, the next request is judged with the new role.

## 4. Customers (CUST)

### CUST-001: Registration and verification

`Mandatory · MVP`

- **Description:** A customer registers with name and email; the email must be verified before they can
  see their ticket history. Registration can happen during the first ticket submission.
- **Restrictions:** One account per email, compared case-insensitively.
- **Edge cases:** Registering with a known email signs in or recovers the existing account; it never
  creates a second one.
- **Acceptance:** No two customers share an email in any letter case; an unverified customer cannot
  list tickets.

### CUST-002: Access needs identity, not a reference

`Mandatory · MVP`

- **Description:** A ticket is reachable only by its signed-in requester (or staff). Knowing a ticket
  reference grants nothing.
- **Acceptance:** A valid reference without the right sign-in shows nothing.

### CUST-003: Profile

`Mandatory · MVP`

- **Description:** Name, email and preferred language only (data minimisation). Staff may correct a
  profile with a reason; email changes need verification of the new address and notify the old one.
- **Acceptance:** The profile holds no other personal field.

### CUST-004: Ticket history

`Mandatory · MVP`

- **Description:** A customer sees all and only their own tickets, newest first, with customer-facing
  status (13.1) and public messages.
- **Acceptance:** The list equals the tickets where they are requester.

### CUST-005: Blocking

`Mandatory · MVP`

- **Description:** An Admin may block a customer for abuse or spam, with a reason.
- **Behavior:** A blocked customer cannot submit or reply; staff may still finish their open tickets.
- **Acceptance:** A blocked customer's writes are refused and audited.

### CUST-006: Guest submission

`Configurable (default: off) · Later`

- **Description:** Optional: submit without an account; the reply link is sent to the given email and
  must be confirmed before anything is shown.

## 5. Tickets (TICKET)

### TICKET-001: Creation

`Mandatory · MVP` (inbound email: `Later`)

- **Description:** A customer submits a ticket on the support portal; an Agent or Admin may create one
  for a customer (e.g. after a phone call).
- **Conditions:** Required fields valid (TICKET-003); customer not blocked.
- **Behavior:** Status **New**, a reference (TICKET-002), SLA clocks start (SLA-002), routing and
  assignment run (section 6), the customer gets an acknowledgement with the reference.
- **Restrictions:** The customer never chooses a team, an agent or the priority. A ticket created by
  staff records both the staff author and the customer.
- **Edge cases:** The same customer submits the same text twice within 10 minutes: the second is shown
  to staff as a probable duplicate (TICKET-010), never dropped.
- **Acceptance:** Every successful submission returns a reference at once and lands with an agent or in
  a team queue.

### TICKET-002: Reference

`Mandatory · MVP`

- **Description:** A short reference with a growing number (e.g. `T-1042`), assigned at creation, never
  changed or reused.
- **Restrictions:** Not a secret; grants no access (CUST-002).
- **Acceptance:** No two tickets share a reference.

### TICKET-003: Required fields

`Mandatory · MVP`

| Field       | Rule                                                                |
| ----------- | ------------------------------------------------------------------- |
| Subject     | 1–200 characters after trimming.                                    |
| Description | 1–20 000 characters after trimming.                                 |
| Category    | One of the active categories; "Other / not sure" is always offered. |
| Attachments | Optional (TICKET-011).                                              |

- **Behavior:** Invalid input is refused with the field named; nothing is created.
- **Acceptance:** Each rule broken alone refuses the submission with that field named.

### TICKET-004: Categories and teams

`Configurable · MVP`

- **Description:** Admins define customer-facing categories, each linked to one team. "Other / not sure"
  goes to a default team. With a single team, every category goes to it.
- **Behavior:** Staff may change the category; if the team changes, the ticket re-enters assignment.
- **Restrictions:** Category names must not reveal teams or staff.
- **Acceptance:** Every ticket has exactly one category and, through it, one team.

### TICKET-005: Priority and status

`Mandatory · MVP`

- **Description:** Every ticket has exactly one priority (section 7) and one status (section 13).
- **Acceptance:** No ticket exists without both.

### TICKET-006: Updates

`Mandatory · MVP`

- **Description:** After submission the customer adds information only by replying; subject and
  description never change (P5). Staff changes to category, priority, team or assignee are recorded
  with author, time, old and new value; lowering a priority needs a reason.
- **Acceptance:** The subject and description shown equal what was submitted.

### TICKET-007: Assignment is internal

`Mandatory · MVP`

- **Description:** Assignee and team are never shown to the customer (P1).
- **Acceptance:** No customer output contains an assignee or team, by name or id.

### TICKET-008: Withdrawal

`Mandatory · MVP`

- **Description:** The customer may withdraw a ticket that is New, Open or Pending. It becomes Closed
  with reason "withdrawn" and is excluded from SLA results (SLA-008).
- **Restrictions:** A Resolved ticket cannot be withdrawn; the customer confirms or reopens instead.
- **Acceptance:** A withdrawn ticket is Closed with that reason and counts in no SLA result.

### TICKET-009: Reopen and close

`Configurable (reopen window and auto-close: 7 days) · MVP`

- **Description:** While Resolved and inside the window, the customer may reject the resolution or reply:
  the ticket returns to Open and its reopen count rises by one. If the customer confirms, or the window
  ends, it becomes Closed.
- **Restrictions:** Closed is final; a customer reply to a Closed ticket starts a new ticket linked to it.
- **Edge cases:** Reopened more than 3 times: flagged Escalated for the Admins.
- **Acceptance:** A reply inside the window reopens; after closure it creates a linked new ticket.

### TICKET-010: Duplicates

`Mandatory · MVP` (merging: `Later`)

- **Description:** An Admin may close ticket B as a duplicate of ticket A of the same customer. B keeps
  its history and links to A; the customer is told to continue on A.
- **Behavior:** B is excluded from SLA results (SLA-008).
- **Acceptance:** After marking, the customer sees both histories and one open ticket.

### TICKET-011: Attachments

`Mandatory · MVP` (limits configurable)

- **Description:** Customers and staff may attach files (proposed: 10 MB per file, 20 per ticket; types
  D-03). Files are checked for malware before download; infected files are quarantined and shown as
  "removed for safety".
- **Restrictions:** A file has its message's visibility: files on internal notes never reach customers.
- **Acceptance:** Over-limit, blocked-type or infected files are never downloadable.

### TICKET-012: History

`Mandatory · MVP`

- **Description:** Each ticket has an append-only timeline of messages, notes, status, assignment and
  priority changes, SLA events and automatic actions, with actor (person, customer or "automation") and
  time.
- **Restrictions:** Only legal erasure (SEC-005) and redaction (COMM-006) change content, leaving a marker.
- **Acceptance:** Any past state of a ticket can be reconstructed from its timeline.

## 6. Assignment (ASSIGN)

The customer is never asked to choose, and never told, who handles their ticket (P1).

### ASSIGN-001: Routing

`Mandatory · MVP`

- **Description:** A new ticket goes to its category's team, then to one agent of that team, or waits in
  the team queue.
- **Acceptance:** Every ticket belongs to exactly one team at all times.

### ASSIGN-002: Who can receive a ticket

`Mandatory · MVP`

An agent is **available** for automatic assignment when all hold:

1. Active member of the ticket's team.
2. Status set to **Available** (agent-set: Available, Away, Off).
3. Inside business hours.

- **Acceptance:** Automatic assignment never picks an agent failing any condition.

### ASSIGN-003: Round-robin

`Configurable (default: on per team) · MVP` (least-loaded: `Later`)

- **Description:** Each team keeps its agents in a fixed order; the next ticket goes to the next available
  agent after the last one assigned. Unavailable agents are skipped. A team may turn automatic
  assignment off and work from its queue only.
- **Acceptance:** With N available agents, N consecutive tickets go to N different agents.

### ASSIGN-004: Unassigned queue

`Mandatory · MVP`

- **Description:** If nobody is available (night, holidays, everyone Away), the ticket waits in the team
  queue, visible to the team and the Admins.
- **Behavior:** Assignment retries when an agent becomes available and at the start of business hours,
  oldest SLA due time first. If a ticket is still unassigned at half its first-response target, Admins
  are alerted; at the full target it is flagged Escalated (SLA-005).
- **Acceptance:** No ticket waits past its first-response target without an Admin alert on record.

### ASSIGN-005: Reassignment

`Mandatory · MVP`

| Trigger                                                             | Result                                                                    |
| ------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| Agent deactivated or removed from the team                          | Their open tickets re-enter assignment.                                   |
| Agent sets Off                                                      | Their New and Open tickets re-enter assignment; Pending ones stay (D-04). |
| Category changed to another team                                    | Re-enters assignment in the new team.                                     |
| Admin reassigns                                                     | Goes to the chosen agent.                                                 |
| No staff action on an Open ticket for 1 business day (configurable) | Assignee and Admins alerted (no automatic move).                          |

- **Acceptance:** Every reassignment has a reason in the timeline.

### ASSIGN-006: Back to the same agent

`Configurable (default: on) · MVP`

- **Description:** A reopened ticket or a linked follow-up goes back to the agent who resolved it if they
  are available; otherwise normal assignment.
- **Acceptance:** Reopening with that agent available assigns to them directly.

## 7. Priority (PRIO)

### PRIO-001: Levels

`Mandatory · MVP`

| Priority | Meaning                                                                                    |
| -------- | ------------------------------------------------------------------------------------------ |
| Critical | The customer cannot use the product or service at all, or there is a safety or legal risk. |
| High     | An important function is broken with no workaround.                                        |
| Medium   | Broken or wrong, with a workaround.                                                        |
| Low      | Question, request or cosmetic issue.                                                       |

- **Acceptance:** Every ticket has one of the four.

### PRIO-002: Initial priority

`Configurable · MVP`

- **Description:** A new ticket takes its category's default (Medium if unset). The customer never sets
  priority.
- **Acceptance:** A new ticket's priority equals its category default.

### PRIO-003: Changes

`Mandatory · MVP`

- **Description:** Agents may raise priority and lower it to Medium or Low; lowering from Critical or High
  needs an Admin. Every change has a reason. Critical tickets with nobody available alert the Admins at once.
- **Acceptance:** An Agent lowering a High ticket is refused.

### PRIO-004: Escalation is not priority

`Mandatory · MVP`

- **Description:** Waiting never raises priority by itself. A ticket at risk is flagged **Escalated**
  (13.3) instead.
- **Purpose:** Priority describes the problem; escalation describes our lateness. Mixing them hides breaches.
- **Acceptance:** Priority changes only by a person.

### PRIO-005: Hidden from customers

`Mandatory · MVP`

- **Acceptance:** No customer output shows a priority.

## 8. SLA (SLA)

The **first-response target** measures how fast the company first answers; the **resolution target**
how fast it fixes the problem.

### SLA-001: Targets

`Configurable · MVP` (D-01 for the values)

| Priority | First response   | Resolution       |
| -------- | ---------------- | ---------------- |
| Critical | 2 business hours | 1 business day   |
| High     | 4 business hours | 2 business days  |
| Medium   | 1 business day   | 5 business days  |
| Low      | 2 business days  | 10 business days |

- **Restrictions:** A resolution target is never shorter than its first-response target.
- **Acceptance:** Every ticket's targets are fixed at creation from the values in force then.

### SLA-002: Clocks

`Mandatory · MVP`

| Clock          | Starts          | Stops                        |
| -------------- | --------------- | ---------------------------- |
| First response | Ticket creation | First public reply by staff. |
| Resolution     | Ticket creation | Ticket set to Resolved.      |

- **Restrictions:** The automatic acknowledgement, internal notes and assignment do not count as a response.
- **Acceptance:** Both stop moments can be pointed to in the timeline.

### SLA-003: Business hours and holidays

`Configurable · MVP`

- **Description:** Clocks run only inside business hours (e.g. Mon–Fri 09:00–18:00) and not on listed
  holidays. A ticket created outside hours starts counting at the next opening.
- **Acceptance:** A 4-hour target on a ticket created Friday 17:00 (hours 09–18) is due Monday 12:00.

### SLA-004: Pause while waiting for the customer

`Mandatory · MVP`

- **Description:** The resolution clock pauses while the ticket is **Pending**; the first-response clock
  never pauses.
- **Acceptance:** Time in Pending is excluded from resolution time.

### SLA-005: Warning, breach, escalation

`Mandatory · MVP` (warning threshold configurable)

| Moment               | Action                                                             |
| -------------------- | ------------------------------------------------------------------ |
| 75% of a target used | Warn the assignee (or the team queue).                             |
| 100% (breach)        | Record the breach once; alert assignee and Admins; flag Escalated. |

- **Restrictions:** Internal only; customers are never told.
- **Acceptance:** A ticket passing a target has exactly one breach record for it.

### SLA-006: Breaches are permanent

`Mandatory · MVP`

- **Description:** A breach is never removed by later events; an Admin may mark it excused with a
  reason, keeping the record.
- **Acceptance:** Reports count every breach; excused ones separately.

### SLA-007: Changes during a ticket

`Mandatory · MVP` (D-02)

- **Description:** Editing targets or hours affects only new tickets. Changing a ticket's priority
  switches to the new targets measured from creation; proposed: if the new target is already exceeded,
  the breach is recorded at the change.
- **Acceptance:** Editing targets never moves an existing ticket's due time.

### SLA-008: Exclusions

`Mandatory · MVP`

- **Description:** Withdrawn, spam and duplicate tickets are excluded from SLA results.
- **Acceptance:** Excluded tickets appear in no compliance figure.

## 9. Communication (COMM)

### COMM-001: Public replies and internal notes

`Mandatory · MVP`

- **Description:** Staff write public replies (customer reads them) or internal notes (staff only).
  Customer messages are always public.
- **Restrictions:** Internal notes and their files never reach customers in any form.
- **Acceptance:** No customer channel ever contains an internal note.

### COMM-002: Customer replies

`Mandatory · MVP`

- **Description:** The customer may reply on any ticket that is not Closed.
- **Behavior:** On Pending it moves to Open; on Resolved inside the window it reopens (TICKET-009); on
  Closed it starts a linked new ticket. The assignee or team queue is notified.
- **Acceptance:** Each reply produces exactly the listed transition.

### COMM-003: How replies are signed

`Configurable (default: company support name) · MVP`

- **Description:** Replies reach the customer signed "<Company> Support". Admins may switch to "agent
  first name + <Company> Support".
- **Purpose:** P1: the customer talks to the company; staff privacy.
- **Restrictions:** Never shown: staff surname, personal email, phone, team, or that the handling agent
  changed. All emails come from the support address.
- **Acceptance:** In the default mode, nothing tells the customer which person replied.

### COMM-004: Notifications

`Mandatory · MVP`

- **Customers** get: acknowledgement with reference; public replies; "we need something from you"
  (Pending); resolved with confirm/reopen; closed; one feedback request. Never internal events.
- **Staff** get: assigned to me; customer replied on my ticket; mention in a note; SLA warning or breach;
  Escalated (Admins); nobody available (Admins).
- **Acceptance:** Each event notifies exactly the people in matrix 14.5.

### COMM-005: Email and abuse limits

`Mandatory · MVP` (reply by email: `Later`)

- **Description:** Notifications go by email from the support address with the ticket reference.
  Submissions and replies are rate-limited per customer (proposed: 5 new tickets per hour). Admins may
  close tickets as spam (no customer email, excluded from figures).
- **Edge cases:** Bounced email: the customer's address is flagged; the portal still works.
- **Acceptance:** Over the limit, nothing is created.

### COMM-006: Redaction

`Mandatory · MVP`

- **Description:** Messages are never edited or deleted. If a customer shares sensitive data (a card
  number, a password), an Admin may redact it; a "redacted" marker stays and the action is audited.
- **Acceptance:** The redacted value cannot be recovered by any user.

## 10. Automation (AUTO)

`Mandatory · MVP` (thresholds configurable; custom rules: `Later`)

| ID       | Trigger                              | Action                                                                          |
| -------- | ------------------------------------ | ------------------------------------------------------------------------------- |
| AUTO-001 | New ticket                           | Acknowledge customer; route; assign; start clocks.                              |
| AUTO-002 | Ticket assigned                      | Notify the agent. Customer: nothing.                                            |
| AUTO-003 | Nobody available                     | Keep in queue; retry; alert Admins per ASSIGN-004.                              |
| AUTO-004 | 75% of a target                      | Warn per SLA-005.                                                               |
| AUTO-005 | Target passed                        | Record breach; alert; flag Escalated.                                           |
| AUTO-006 | Customer reply                       | Transition per COMM-002; notify assignee or queue.                              |
| AUTO-007 | Open ticket untouched 1 business day | Alert assignee and Admins.                                                      |
| AUTO-008 | Pending with no customer reply       | Reminder after 3 business days; set Resolved with reason "no response" after 7. |
| AUTO-009 | Ticket resolved                      | Tell the customer with confirm/reopen; start the auto-close timer.              |
| AUTO-010 | Ticket closed normally               | One feedback request (1–5 and optional comment), open for 14 days.              |

- **Restrictions:** Automatic actions appear in the timeline with actor "automation" and never send
  customers internal data.
- **Acceptance:** Each trigger produces exactly its listed action, once.

## 11. Security and audit (SEC)

### SEC-001: Access control

`Mandatory · MVP`

- **Description:** Every action is checked against the person's role and teams at the time of the action.
  Anything not allowed is refused; tickets outside someone's reach are answered as "not found".
- **Acceptance:** Each role gets exactly the results in matrix 14.2.

### SEC-002: Sign-in

`Mandatory · MVP`

- **Description:** Staff and customers sign in with a verified email and a password of at least 12
  characters. Repeated failures lock the account for a while without revealing whether the email exists.
  Admins must use a second factor (D-05).
- **Acceptance:** A wrong password and an unknown email give the same answer.

### SEC-003: Privacy

`Mandatory · MVP`

- **Description:** Staff reach customer data only through tickets they can see; customer data is used only
  for support. Customer lists can be exported by Admins only, audited.
- **Acceptance:** An Agent reaches a customer only via a visible ticket.

### SEC-004: Audit log

`Mandatory · MVP`

- **Recorded:** sign-ins and failures, staff and role changes, settings changes, exports, redactions,
  blocks, erasures, excused breaches. Append-only, Admin-readable, kept 1 year (D-06), never contains
  passwords or message bodies.
- **Acceptance:** Each listed event creates one entry with actor, time and target.

### SEC-005: Retention and erasure

`Configurable · MVP` (D-07)

- **Description:** Closed tickets are kept for a set period (proposed 3 years), then erased; anonymous
  totals may stay. A customer may ask for a copy or erasure of their data; on erasure their personal data
  and messages are removed or anonymised, leaving placeholders so totals stay correct.
- **Acceptance:** After erasure nothing identifies the person; report totals are unchanged.

## 12. Reporting (REP)

`Mandatory · MVP` (exports and trends: `Later`)

| Metric                      | Definition                                                                                    |
| --------------------------- | --------------------------------------------------------------------------------------------- |
| Tickets created             | Created in the period, excluding spam.                                                        |
| Open tickets                | New, Open or Pending at a given moment.                                                       |
| Resolved tickets            | Reached Resolved in the period, counted once even if reopened.                                |
| Overdue tickets             | Open tickets past a target at a given moment.                                                 |
| Average first response time | Business time to first public reply, tickets first answered in the period, excluding SLA-008. |
| Average resolution time     | Business time to Resolved, Pending excluded, for tickets resolved in the period.              |
| SLA compliance              | Per target: met ÷ decided in the period, excluding SLA-008; excused breaches shown apart.     |
| Agent workload              | Open tickets per agent now; tickets assigned per agent in the period.                         |
| Customer satisfaction       | Share of 4–5 ratings among ratings in the period, with response rate.                         |
| By category                 | The above per category.                                                                       |

- **Scope:** Admins see everything; Agents see their own numbers only.
- **Acceptance:** The same metric for the same scope and period always shows the same value.

## 13. Ticket lifecycle

### 13.1 Statuses

| Status   | Meaning                                                         | Customer sees          | Resolution clock |
| -------- | --------------------------------------------------------------- | ---------------------- | ---------------- |
| New      | Created; no staff answer or work yet.                           | Received               | Running          |
| Open     | Staff are working on it.                                        | In progress            | Running          |
| Pending  | Staff asked the customer for something.                         | Waiting for your reply | Paused           |
| Resolved | Staff believe it is solved; the customer may confirm or reopen. | Solved, please confirm | Stopped          |
| Closed   | Final.                                                          | Closed                 | Stopped          |

**Reopened and Escalated are not statuses.** A reopened ticket is simply worked again (Open); an
escalated ticket can be New, Open or Pending at the same time. So **Reopened** is an event with a count
(TICKET-009) and **Escalated** is a flag (13.3). Making them statuses would force choices like "Escalated
or Pending?" and break the SLA pause.

### 13.2 Transitions

| #  | From                         | To       | Who                                                             | Condition                                       | Automatic actions                                  |
| -- | ---------------------------- | -------- | --------------------------------------------------------------- | ----------------------------------------------- | -------------------------------------------------- |
| T1 | —                            | New      | Customer; staff for a customer                                  | TICKET-001                                      | AUTO-001                                           |
| T2 | New                          | Open     | Assignee, Admin                                                 | First public reply or "start work"              | First-response clock stops on the reply.           |
| T3 | New, Open                    | Pending  | Assignee, Admin                                                 | Sent with a public message asking for something | Resolution clock pauses; customer told.            |
| T4 | Pending                      | Open     | Customer (by replying), staff                                   | —                                               | Clock resumes; assignee notified.                  |
| T5 | New, Open, Pending           | Resolved | Assignee, Admin; automation (AUTO-008)                          | Sent with a public resolution message           | Clock stops; customer told; window starts.         |
| T6 | Resolved                     | Open     | Customer; Admin                                                 | Inside the reopen window                        | Reopen count +1; clock resumes (D-08); ASSIGN-006. |
| T7 | Resolved                     | Closed   | Customer (confirms); automation                                 | Confirmed, or window ended                      | Feedback request.                                  |
| T8 | New, Open, Pending, Resolved | Closed   | Customer (withdraw, not from Resolved); Admin (spam, duplicate) | Reason recorded                                 | Excluded from SLA; customer told except spam.      |

**Forbidden:** anything out of Closed (a linked new ticket instead); anything back to New; Resolved →
Pending (reopen first); staff closing without resolving except as spam or duplicate; customers setting
Open, Pending or Resolved directly.

**Invalid attempts:** nothing changes; the person is told the current status and what is allowed.
**Simultaneous changes:** the second change, made on an outdated view, is refused and asks for a reload.

### 13.3 Escalated flag

- **Set by:** an SLA breach, a Critical ticket with nobody available, an unassigned ticket past its
  first-response target, more than 3 reopens, or an Admin.
- **Cleared by:** an Admin with a note, or automatically on Resolved. Staff-only, recorded in the timeline.

## 14. Supporting specifications

### 14.1 Workflow

1. The customer picks a customer-facing category and describes the problem.
2. The ticket is New with a reference; the customer is acknowledged; clocks start.
3. The category's team gets it; round-robin assigns an available agent, or it waits in the team queue.
4. The agent answers (Open) or asks for something (Pending, resolution clock paused).
5. Staff coordinate in internal notes; replies reach the customer signed by the company.
6. Warnings and breaches alert the assignee and Admins and flag the ticket Escalated.
7. Staff resolve with an explanation; the customer confirms or reopens within the window.
8. Otherwise it closes automatically; one feedback request follows.
9. The timeline keeps everything; reports summarise it; retention erases it later.

### 14.2 Permission matrix

| Action                                | Admin  | Agent                        | Customer         |
| ------------------------------------- | ------ | ---------------------------- | ---------------- |
| Settings, staff, roles                | ✓      | ✗                            | ✗                |
| See tickets                           | All    | Assigned + own teams' queues | Own              |
| Create ticket                         | ✓      | ✓ (for a customer)           | ✓                |
| Public reply / internal note          | ✓ / ✓  | ✓ / ✓                        | Reply only       |
| Assign / reassign                     | ✓      | Take from own queue          | ✗                |
| Raise / lower priority                | ✓ / ✓  | ✓ / to Medium or Low only    | ✗                |
| Resolve                               | ✓      | Assigned only                | ✗                |
| Confirm / reopen inside window        | Reopen | ✗                            | ✓                |
| Withdraw                              | ✗      | ✗                            | ✓ (not Resolved) |
| Close as spam / duplicate             | ✓      | Suggest only                 | ✗                |
| Excuse breach, block customer, redact | ✓      | ✗                            | ✗                |
| Reports                               | All    | Own numbers                  | ✗                |
| Audit log                             | ✓      | ✗                            | ✗                |

### 14.3 Status transition matrix

| From \ To | New | Open | Pending | Resolved | Closed |
| --------- | --- | ---- | ------- | -------- | ------ |
| New       | —   | T2   | T3      | T5       | T8     |
| Open      | ✗   | —    | T3      | T5       | T8     |
| Pending   | ✗   | T4   | —       | T5       | T8     |
| Resolved  | ✗   | T6   | ✗       | —        | T7, T8 |
| Closed    | ✗   | ✗    | ✗       | ✗        | —      |

### 14.4 Assignment decision rules

1. Team = the category's team.
2. Reopened or follow-up, and the previous agent is available → that agent. Stop.
3. Team uses automatic assignment and an agent is available (ASSIGN-002) → next in round-robin. Stop.
4. Otherwise → team queue; retry later; alert Admins at half the first-response target (Critical: at once).
5. Queues are worked oldest SLA due time first, Critical first.

### 14.5 Notification matrix

| Event                    | Customer     | Assignee        | Admins                            |
| ------------------------ | ------------ | --------------- | --------------------------------- |
| Ticket created           | ✓ ack        | —               | —                                 |
| Ticket assigned          | —            | ✓               | —                                 |
| Nobody available         | —            | —               | ✓ (half target; Critical at once) |
| Customer replied         | —            | ✓ (else team)   | —                                 |
| Public staff reply       | ✓            | —               | —                                 |
| Pending / reminder       | ✓            | —               | —                                 |
| Resolved / Closed        | ✓ (not spam) | —               | —                                 |
| Duplicate                | ✓            | ✓               | —                                 |
| Mention in note          | —            | mentioned staff | —                                 |
| SLA warning              | —            | ✓ (else team)   | —                                 |
| SLA breach / Escalated   | —            | ✓               | ✓                                 |
| Untouched 1 business day | —            | ✓               | ✓                                 |
| Feedback request         | ✓ once       | —               | —                                 |

### 14.6 Exceptions

| Situation                               | Behavior                                                |
| --------------------------------------- | ------------------------------------------------------- |
| Nobody available                        | Queue, retries, Admin alert (ASSIGN-004).               |
| Agent deactivated with open tickets     | Reassigned (ASSIGN-005).                                |
| Customer email bounces                  | Address flagged; portal still works.                    |
| Duplicate submission                    | Shown as probable duplicate (TICKET-001).               |
| Infected attachment                     | Quarantined, shown as removed.                          |
| Two staff change a ticket at once       | Second refused, reload asked.                           |
| Invalid transition                      | Refused, explained, nothing changes.                    |
| Customer asks for erasure, tickets open | Erased after closure unless law requires sooner (D-07). |

### 14.7 Invariants

| ID     | Never violated                                                                                                                    |
| ------ | --------------------------------------------------------------------------------------------------------------------------------- |
| INV-01 | Every ticket has exactly one requester, category, team, priority and status.                                                      |
| INV-02 | Every open ticket has an assignee or sits in a team queue visible to Admins.                                                      |
| INV-03 | Customers never see internal notes, staff identity (default mode), teams, assignment, priority, SLA timers or the Escalated flag. |
| INV-04 | References are unique and never reused.                                                                                           |
| INV-05 | Closed is final.                                                                                                                  |
| INV-06 | Timelines and messages are append-only, except erasure and redaction, which leave markers.                                        |
| INV-07 | Each target breaches at most once per ticket; breaches are never deleted.                                                         |
| INV-08 | At least one active Admin exists.                                                                                                 |
| INV-09 | Settings changes never rewrite the past.                                                                                          |

## 15. Open business decisions

| ID   | Decision                                                  | Proposed default                                        |
| ---- | --------------------------------------------------------- | ------------------------------------------------------- |
| D-01 | SLA target values                                         | Table in SLA-001.                                       |
| D-02 | Priority raised past an already-exceeded target           | Breach at the change.                                   |
| D-03 | Allowed attachment types                                  | Documents, images, PDF, text, archives; no executables. |
| D-04 | Agent goes Off: do Pending tickets move?                  | No, unless Off longer than 2 business days.             |
| D-05 | Second factor for which roles?                            | Admins required; others optional.                       |
| D-06 | Audit log retention                                       | 1 year.                                                 |
| D-07 | Ticket retention after closure; erasure with open tickets | 3 years; erase after closure.                           |
| D-08 | Reopen: resume or restart the resolution clock?           | Resume.                                                 |
| D-09 | Do customers need to see expected response times?         | Later.                                                  |
| D-10 | Guest submission at all?                                  | Off; Later.                                             |

## 16. MVP versus later

| Area          | MVP                                                                                                          | Later                  |
| ------------- | ------------------------------------------------------------------------------------------------------------ | ---------------------- |
| Roles         | Admin, Agent, Customer; immediate role changes                                                               | —                      |
| Customers     | Registration, verification, history, blocking                                                                | Guest submission       |
| Tickets       | Portal and staff-created tickets, references, lifecycle, withdraw, reopen, duplicates, attachments, timeline | Inbound email, merging |
| Assignment    | Category → team, round-robin, queue, reassignment, back-to-same-agent                                        | Least-loaded, skills   |
| SLA           | Targets per priority, business hours, holidays, pause, warning, breach                                       | Per-team hours         |
| Communication | Public/internal, company-signed replies, email notifications, limits, redaction                              | Reply by email         |
| Automation    | AUTO-001 to AUTO-010                                                                                         | Custom rules           |
| Reporting     | Metrics in section 12                                                                                        | Exports, trends        |

## 17. Gap against the current PoC

The PoC already matches the three roles and the thin-event, no-addresses-in-logs approach. Gaps:

| Gap                                                                               | Rules                  |
| --------------------------------------------------------------------------------- | ---------------------- |
| ~~The tickets API returns `assignee_id` to customers.~~ Closed in scope 6.        | P1, TICKET-007, INV-03 |
| Statuses are open, in_progress, resolved, closed: no New, no Pending (SLA pause). | 13.1                   |
| ~~UUIDs only, no short reference.~~ Closed in scope 6 (`T-1042`).                 | TICKET-002             |
| No categories, teams, priorities, messages, internal notes or SLA yet.            | Sections 5–9           |
| Assignment is manual; no round-robin or queue.                                    | Section 6              |
| Ticket permissions use the role in the access token, up to 15 minutes old.        | ROLE-006               |

These are planning notes; the build order decides if and when each is addressed.
