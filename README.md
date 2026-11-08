# Synixe Apps

The main apps that run Synixe

You are welcome to read, not copy directly, the source code for inspiration for your own community projects.

## Features

### Mission Management

*Some features come from SynixeContractors/Missions*

- Validate missions
    - Naming standards
    - Respawn & Spectator configured
    - Briefing configured
    - Other group specific rules
- Generate `.pbo` files for missions
    - For regular missions
    - Gernerate missions for multiple maps from a single mission file
    - Automatically loaded onto all servers
- Schedule missions
- Provide an up-to-date iCal for Google Calendar and other calendar apps
- Post in a #schedule channel and track RSVPs
- Start the mission on the appropriate server before it starts
- Track attendance
- After action reports are validated
- Load missions and restart servers with Discord slash commands

### Persistent Gear

- Loadout Tracking
    - Loadouts are persistent across missions
    - CBA Extended Loadouts (ACE Earplugs, GRAD Sling Helmet)
- Bank System
    - Players earn money for completing missions
    - Players can buy gear from the shop
    - Players can transfer money to each other
    - Automated payments from valid after action reports
- Garage System
    - Players can buy vehicles from the shop using group funds
    - Players can modify vehicles, attaching weapons
    - Players can retrieve and store vehicles in the garage
    - State of vehicles is tracked
        - Fuel
        - Damage
        - Ammo
        - ACE Cargo

## Persistent Campaign Maps

- Tracks all objects
    - Damage
    - Inventory
    - ACE Cargo
- Tracks all groups
    - Waypoints
- Tracks all units
    - Damage
    - Inventory
- Advanced Equipment (AE3) support, when AE3 is loaded
    - Saves the state of computers (including power), networks, and flash drives
    - Each saved device is keyed by its own Crate ID, not scoped to a campaign, so it keeps its state when carried between missions (e.g. in a laptop's inventory item)
    - A device recreated with a previously-known ID (redeployed from inventory, or recreated after a restart) is automatically restored to its last saved state
    - Zeus module "Restore AE3 Device" applies a saved device's state to the attached object, given its saved ID
    - Device state is stored separately from object data, so object sync can't remove it
    - Large states (e.g. a big filesystem) are sent in chunks to stay under the callExtension size limit
    - AE3's own packed-item buffers (laptops and flash drives carried in inventory) are mirrored to the database and rebuilt at mission start, so a packed device can still be redeployed with its saved state after a restart

### Certifications

- Instructors can certify students
- Members can see available certifications
- Expiry notifications
- Certifications can unlock gear in the shop
- Certifications can give free gear the first time they are completed
- ACE roles are automatically assigned based on certifications
