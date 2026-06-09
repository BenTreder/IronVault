# IronVault GUI Design Brief

Status: Phase 15

This document captures the agreed frontend direction before building the desktop UI.

## Collaboration rule

The IronVault frontend should be built collaboratively with Ben involved in the design process.

Before making major frontend decisions, ask Ben about:

- styling
- layout
- button text
- dashboard wording
- color choices
- tone
- onboarding flow
- warning language
- empty states
- restore safety screens

Do not silently make major design choices.

## Visual direction

Theme support:

- dark mode
- light mode

Primary accent:

- Iron Orange

Suggested Iron Orange starting point:

- #f97316

The final color can be adjusted after seeing it in the app.

## App personality

IronVault should feel like a premium friendly utility.

It should feel:

- familiar
- calm
- user friendly
- protective
- polished
- modern
- premium
- easy to understand

It should not feel:

- too technical
- too gamer-like
- too corporate
- too goofy
- too scary
- too bland

## Tone

Tone should be serious with a bit of fun.

Good direction:

- Vault door closed. Everything looks safe.
- Backup sealed.
- Restore map ready.
- Every vault piece is accounted for.
- No guessing, no stomping, no chaos.

Avoid:

- oopsie
- maybe safe
- probably fine
- scary disaster wording
- confusing developer language

## Dashboard headline direction

Recommended main dashboard headline:

Your vault is protected

Recommended subtext:

Vault door closed. Everything looks safe.

Why this works:

- It feels premium.
- It is clear.
- It sounds safe.
- It has a small amount of IronVault personality.
- It does not make backup safety sound like a joke.

## Dashboard layout direction

Suggested first dashboard layout:

1. Top header
   - IronVault logo or name
   - theme toggle
   - settings button

2. Hero status card
   - headline
   - vault health
   - last backup state
   - strong primary action

3. Dashboard cards
   - Vault Health
   - Last Backup
   - Snapshots
   - Vault Size

4. Recent activity or snapshot list
   - simple, readable, not cluttered

5. Safety panel
   - restore conflicts
   - verify warnings
   - clear next step

## Navigation

Suggested sidebar or top navigation:

- Dashboard
- Backup
- Restore
- Snapshots
- Settings

Keep navigation simple at first.

## Key screens

### Dashboard

Purpose:

Show whether the vault is safe and what the user should do next.

Main copy:

Your vault is protected

Subcopy:

Vault door closed. Everything looks safe.

Primary action ideas:

- Run Backup
- Check Vault
- Restore Files

### Backup

Purpose:

Let users choose what to back up and run a backup safely.

Tone:

- simple
- clear
- reassuring

Possible copy:

Choose what goes in the vault.

### Restore

Purpose:

Preview before restore, show conflicts clearly, never surprise the user.

Rules:

- Always show restore plan before restore.
- Never hide conflicts.
- Do not make overwrite the default.
- Skip existing mode can be offered as a safe option.

Possible copy:

Restore map ready.

Conflict copy:

Vault door closed. IronVault found files already sitting there.

### Snapshots

Purpose:

Show previous backups in a friendly, clear list.

Possible copy:

Vault shelves

Alternative safer copy:

Backup snapshots

## Current GUI bridge commands

The GUI can use these CLI commands now:

- ironvault info --repo ./repo --json
- ironvault snapshots --repo ./repo --json
- ironvault verify --repo ./repo --json
- ironvault restore-plan --repo ./repo --snapshot latest --target ./restore --json
- ironvault restore --repo ./repo --snapshot latest --target ./restore --if-exists skip

## Creator attribution

The frontend should include tasteful creator attribution.

Recommended footer copy:

Made by Ben Treder

Website link:

https://bentreder.com

Display text:

BenTreder.com

Placement ideas:

- Footer area
- About screen
- Settings screen
- Small bottom-left or bottom-right app shell credit

The attribution should feel premium and subtle, not distracting.

Suggested UI wording:

Made by Ben Treder



## Frontend build rule

Before changing the actual frontend UI, inspect the current Vue and Tauri files.

After inspection, build in small visible steps:

1. theme tokens
2. base layout
3. dashboard hero
4. dashboard cards
5. navigation
6. real JSON command bridge
7. restore preview screen

Each visual step should be easy for Ben to review.
