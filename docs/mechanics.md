# Mechanics/Ultras

This feature allows you to setup visual alerts for ultras. The code automatically watches the game and tells you exactly when to use important skills such as Decay/Taunt, or even simple things like Zones.

The `mechanics.json` holds a bunch of presets for different ultras, for example Ultra Darkon, Champion Drakath, etc. You can use the existing options, or write your own custom presets for different bosses. 

## Setting up an Ultra
To get started, open the `mechanics.json` file in a text editor. All the presets are organised by the game's map name.

### 1. Map Name
The very first thing you need is the exact map name of the ultra boss, written in lowercase. For example, if you are fighting Champion Drakath, the label is `"championdrakath"`. Every strategy for that boss goes inside it's brackets, the `[]` indicates a list. 

Example
```json
{
    "championdrakath": []
}
```

### 2. Adding a Strategy
Inside the map name, you make a list of your different stratgies. Each strategy needs a few basic pieces of information.

`name` - The name for this strategy. e.g `"Cdrak Solo Taunt"`
`roles` - A list of classes your team is using, e.g `["LOO", "LR", "DPS", "AP"]`

Example
```json
{
    "championdrakath": [
        {
            "name": "Cdrak Solo Taunt",
            "roles": ["LOO", "LR", "DPS", "AP"]
        }
    ]
}
```

### 3. Setting up triggers/actions
It's basically a way to tell the tool *when* it should indicate, which role it should indicate to, and what the action should be. 

Here are the possible triggers I've added so far, more will be added.

#### Triggers

**Message** - Takes a message prompt, so when the script sees the message. It will indicate the action. E.g `"You will see the truth"` appears and you setup it to taunt, it will tell you to taunt.

Example
```json
{"trigger": {"message": "You will see the truth"}, "role": "LOO", "action": "Taunt"}
```

**Aura** - This takes Aura name, and a timer. The Aura name is the aura you want to check for E.g `"Focus"`, and the Timer is how long **before** the aura expires to begin indicating, e.g `1500` which is 1500ms, equivalent to 1.5s.

So for this example, LOO indicator for Taunt appears when Focus is 1.5s before it expires.
```json
{"trigger": {"aura": {"name": "Focus", "timer": 1500}}, "role": "LOO", "action": "Taunt"}
```

**HP** - This takes a HP value and a timer. It uses DPS to determine how many seconds you are away from the HP value. If timer is less than this value, it indicates you to taunt. This is mostly because DPS can vary the timings of taunts significantly. 

So for this example, when you are 3s away from reaching 18m, it will tell you to taunt.
```json
{"trigger": {"hp": 18000000, "timer": 3000}, "role": "CAV", "action": "Taunt"}
```

#### Actions
Basically what it will indicate for you to do. Possible actions I have are `"Taunt"`, `"Zone"`, `"Decay"` and `"Quixotic"`.

## 4. Choosing a Timeline
So I think this is the most confusing part. Depending on the fight, there's different orders the bosses take, or require. Some fights you need to just react based on an event, whereas other fights you need to follow a strict set order. Hence why I created these.

**Reactives** - These are triggers that are activated based on an event, rather than a set order. E.g a HP threshold check, like Ultra Darkon Quixotic at 4.4m.

**Cycle** - These are triggers that are activated based on a set order. E.g a specific number of taunts, zones, etc, in a set order. Like UltraSpeaker. It will repeat in this order until the fight is over.

**Openings** - These are triggers that are activated based on the opening of a fight, which usually fall back into a cycle. I made this specifically for Speaker cause he sucketh.

You can have Reactives and Cycles as part of one strategy. Here's an example for Ultra Darkon. The LOO Quixotic I put as a HP threshold reactive. So it's basically just waiting for the HP to be 3s out before the script indicates for Quixotic. Similarly, I've created a cycle for LR and LOO loop taunting, by making them check for the Focus Aura. It will repeat in this order till Darkon dies.
```json
{
    "ultradarkon": [
        {
            "name": "LOO LR Loop Taunt",
            "roles": ["LR (t1)", "LOO (t2)", "FB", "DPS"],
            "reactives": [
                {
                    "trigger": { "HP": { "value": 4444444, "timer": 3000 } },
                    "role": "LOO",
                    "action": "Quixotic"
                }
            ],
            "cycle": [
                {
                    "trigger": { "Aura": { "name": "Focus", "timer": 1500 } },
                    "role": "LR (t1)",
                    "action": "Taunt"
                },
                {
                    "trigger": { "Aura": { "name": "Focus", "timer": 1500 } },
                    "role": "LOO (t2)",
                    "action": "Taunt"
                }
            ]
        }
    ]
}
```

## 5. Using these in the script 
This is all usable in the Ultra Mechanics section (hit Enter 4 times). If you aren't in the room, the script will say `No active boss mechanics loaded. Waiting for configuration...`. In order to load the configuration, just rejoin the room. Pressing Left and Right arrow keys select different strategies, and pressing Up and Down keys select your role. If you select your role, the script should indicate when you should taunt/zone/decay etc.

The purpose of this documentation is to help you understand how to write configurations, so that I don't have to write them. You just edit the JSON file, and add the configuration for whatever ultra you want.