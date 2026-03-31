/// Returns a pool of alliterative names for the given species.
pub fn get_names(species: &str) -> &'static [&'static str] {
    match species {
        "duck" => &[
            "Dapper Duck", "Dizzy Daisy", "Daring Domino", "Dreamy Dusk",
            "Dancing Dewdrop", "Dashing Dynamo", "Darling Doodle", "Dusty Diamond",
            "Dainty Dimple", "Devious Dash",
        ],
        "goose" => &[
            "Gallant Goose", "Goofy Gizmo", "Grumpy Garlic", "Gleaming Ghost",
            "Gentle Gust", "Groovy Gadget", "Golden Glimmer", "Gutsy Gumball",
            "Giggly Gem", "Grand Gatsby",
        ],
        "blob" => &[
            "Bouncy Blob", "Bubbly Biscuit", "Bold Blizzard", "Bashful Breeze",
            "Bright Bubbles", "Bumbling Bolt", "Breezy Button", "Blazing Bloom",
            "Blissful Bean", "Brilliant Bop",
        ],
        "cat" => &[
            "Cosmic Cat", "Cunning Claude", "Cozy Cinnamon", "Clever Cricket",
            "Cuddly Cocoa", "Cheeky Charm", "Calm Crescent", "Crafty Cookie",
            "Cool Cascade", "Curious Clover",
        ],
        "dragon" => &[
            "Dazzling Drake", "Dramatic Draco", "Dreamy Dragon", "Daring Dusk",
            "Dark Drizzle", "Dynamic Drift", "Defiant Dagger", "Delightful Dawn",
            "Dashing Drake", "Dire Dusk",
        ],
        "octopus" => &[
            "Outgoing Ollie", "Ornate Oracle", "Oceanic Opal", "Odd Orbit",
            "Optimistic Otto", "Opulent Onyx", "Original Opus", "Outstanding Oak",
            "Organic Olive", "Overjoyed Orion",
        ],
        "owl" => &[
            "Observant Owl", "Opulent Ozzy", "Orderly Oaken", "Otherworldly Ori",
            "Outstanding Olive", "Old Oracle", "Overcast Opal", "Ornery Oscar",
            "Offbeat Omega", "Odd Orchid",
        ],
        "penguin" => &[
            "Playful Penguin", "Perky Pixel", "Peppy Pebble", "Pristine Pearl",
            "Plucky Prism", "Pleasant Plum", "Proud Pilot", "Puzzled Pickle",
            "Patient Patch", "Peaceful Pine",
        ],
        "turtle" => &[
            "Tranquil Turtle", "Trusty Toffee", "Tiny Thunder", "Tidy Trinket",
            "Tough Topaz", "Tender Twilight", "Timid Tango", "True Timber",
            "Ticklish Truffle", "Toasty Thistle",
        ],
        "snail" => &[
            "Sleepy Snail", "Serene Swirl", "Snappy Spark", "Soft Shimmer",
            "Subtle Storm", "Sweet Sprout", "Savvy Sage", "Silly Stardust",
            "Steady Stream", "Shiny Spiral",
        ],
        "ghost" => &[
            "Gentle Ghost", "Glitchy Glow", "Groovy Garlic", "Gleaming Gossamer",
            "Giddy Glitch", "Grand Gust", "Ghostly Gem", "Graceful Glimmer",
            "Gutsy Glimpse", "Gray Gallop",
        ],
        "axolotl" => &[
            "Anxious Avocado", "Amazing Axel", "Artful Azure", "Awesome Acorn",
            "Adorable Atlas", "Astral Amber", "Agile Arrow", "Amusing Alto",
            "Audacious Ash", "Ancient Aurora",
        ],
        "capybara" => &[
            "Chaotic Cappuccino", "Calm Clover", "Cheerful Cobalt", "Cozy Caramel",
            "Cuddly Cloud", "Charming Coral", "Casual Copper", "Chill Cedar",
            "Chubby Cherry", "Clever Chai",
        ],
        "cactus" => &[
            "Cool Cactus", "Crafty Coral", "Crispy Cedar", "Calm Cobalt",
            "Curious Crystal", "Charming Chip", "Cosmic Clove", "Crunchy Chrome",
            "Cheerful Cider", "Cozy Copper",
        ],
        "robot" => &[
            "Rusty Robot", "Radical Rex", "Rogue Radar", "Rapid Rocket",
            "Reliable Rune", "Rowdy Rebel", "Royal Rivet", "Restless Ratchet",
            "Radiant Relay", "Retro Rover",
        ],
        "rabbit" => &[
            "Rascal Rabbit", "Rosy Raindrop", "Rapid Runner", "Relaxed Ripple",
            "Rowdy Rosebud", "Regal Riddle", "Rustic Rainbow", "Radiant Reed",
            "Rolling River", "Rare Rhythm",
        ],
        "mushroom" => &[
            "Magnificent Muffin", "Merry Mosaic", "Mellow Maple", "Mystical Mint",
            "Mirthful Mocha", "Mighty Marble", "Magical Meadow", "Moody Mango",
            "Musical Mist", "Modest Moon",
        ],
        "chonk" => &[
            "Chunky Chestnut", "Cheerful Chomp", "Chubby Charm", "Chill Champion",
            "Chaotic Chunk", "Cuddly Cheese", "Cozy Chipmunk", "Chirpy Chocolate",
            "Calm Cheddar", "Clever Churro",
        ],
        "fox" => &[
            "Foxy Flash", "Fluffy Fern", "Fierce Flame", "Friendly Frost",
            "Feisty Flicker", "Fancy Flint", "Frisky Falcon", "Free Firefly",
            "Fuzzy Fig", "Faithful Fog",
        ],
        "frog" => &[
            "Funky Frog", "Fizzy Fern", "Flashy Flint", "Friendly Fjord",
            "Frosty Frond", "Feisty Flipper", "Fantastic Fig", "Folksy Frost",
            "Fuzzy Fountain", "Frisky Fossil",
        ],
        "bat" => &[
            "Bold Bat", "Bouncy Breeze", "Brave Basil", "Bright Bramble",
            "Bashful Berry", "Blazing Birch", "Blissful Brook", "Brisk Bark",
            "Bumpy Boulder", "Brilliant Bolt",
        ],
        "jellyfish" => &[
            "Jolly Jellybean", "Jazzy Jewel", "Joyful Jasper", "Jumpy Jade",
            "Jubilant Jinx", "Jaunty Jazz", "Jovial Juniper", "Jeweled Jolt",
            "Jiggly Jubilee", "Just Jupiter",
        ],
        "panda" => &[
            "Peaceful Panda", "Plucky Peach", "Playful Pebble", "Patient Plum",
            "Proud Pine", "Pudgy Popcorn", "Peppy Pixel", "Pure Prism",
            "Perky Pistachio", "Pleasant Puff",
        ],
        _ => &["Mystery Companion"],
    }
}
