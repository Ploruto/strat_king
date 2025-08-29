# Game Design - Core Mechanics

## Game Overview
Real-time territorial control strategy game for mobile, focused on 1v1 matches lasting 4-10 minutes.

## Core Mechanics

### Territory System
- **Structures**: Fixed positions on map with connecting paths
- **Towers**: Primary structures that generate mana when owned
- **Base Tower**: Each player's main structure - losing it = elimination
- **Neutral State**: Structures start unowned and can be captured

### Resource Economy
- **Mana**: Core currency that regenerates over time at owned towers
- **Capacity**: Towers have maximum mana storage (no overflow)
- **Regeneration Rate**: Owned towers continuously generate mana

### Troop Deployment
- **Individual Costs**: Each troop type has specific mana cost (e.g., knight costs 8, golem costs 12)
- **Fixed Quantity**: Each troop type deploys in predetermined amounts (e.g., 6x knights, 2x golems)
- **Path-Based Movement**: Troops travel along predefined routes between structures
- **Simple UI**: Drag from start tower to end tower, select available troop type

### Combat System
- **Path Encounters**: Troops fight when they meet on routes
- **Individual Behavior**: Each troop type has unique combat mechanics
- **Survival Continues**: Winners proceed to destination after combat
- **HP-Based**: All troops share health/death mechanic but unique everything else

### Troop Variety
- **Pre-Match Selection**: Players choose 4 troop types before queuing
- **Unique Archetypes**: Examples include runners (goblins), fighters (knights), supporters (mages)
- **Individual Stats & Behavior**: No shared combat system beyond HP

### Victory Condition
- **Base Elimination**: Capture opponent's base tower to win
- **Mana Reduction**: Reduce structure mana below 0 to capture it

### Progression
- **Tower Upgrades**: Improve mana generation rate and capacity using resources

## Future Mechanics (Considered)
- Gold mines for tower upgrade currency
- Taverns for temporary troop upgrades
- Limited-time map events