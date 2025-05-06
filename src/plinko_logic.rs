use rand::Rng;

/// Order: hole1 … hole9  →  index 0 … 8
pub const HOLES: usize = 9;

#[derive(Clone, Copy, Debug)]
pub enum GameMode {
    Easy,
    Medium,
    Hard,
    Extreme,
}

impl TryFrom<i32> for GameMode {
    type Error = ();

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        Ok(match v {
            0 => GameMode::Easy,
            1 => GameMode::Medium,
            2 => GameMode::Hard,
            3 => GameMode::Extreme,
            _ => return Err(()),
        })
    }
}

/* -------------------------------------------------------------------------- */
/*                              TABLES (const)                                */
/* -------------------------------------------------------------------------- */

// (numerator, denominator) – lets us keep everything integer-safe
type Mult = (i64, i64);

const MUL: [[Mult; HOLES]; 4] = [
    /* EASY */
    [(50,1),(10,1),(3,1),(1,1),(1,5),(1,1),(3,1),(10,1),(50,1)],
    /* MEDIUM */
    [(250,1),(34,1),(1,1),(1,2),(1,5),(1,2),(1,1),(34,1),(250,1)],
    /* HARD */
    [(1000,1),(25,1),(1,1),(1,5),(1,5),(1,5),(1,1),(25,1),(1000,1)],
    /* EXTREME */
    [(50_000,1),(4,1),(1,1),(1,5),(1,5),(1,5),(1,1),(4,1),(50_000,1)],
];

const ODDS: [[f64; HOLES]; 4] = [
    /* EASY    */ [0.03,0.3,6.0,22.0,43.35,22.0,6.0,0.3,0.03],
    /* MEDIUM  */ [0.02,0.6,10.0,19.4,40.0,19.4,10.0,0.6,0.02],
    /* HARD    */ [0.013,0.8,10.0,19.4,40.0,19.4,10.0,0.8,0.013],
    /* EXTREME */ [0.0006,0.8,10.0,19.4,40.0,19.4,10.0,0.8,0.0006],
];

/// Run one Plinko roll.  
/// Returns (payout, hole_index 0-8).
pub fn simulate(mode: GameMode, bet: i64) -> (i64, usize) {
    let mi = mode as usize;

    // 1️⃣ choose hole weighted by odds
    let weights = &ODDS[mi];
    let total: f64 = weights.iter().sum();
    let mut r = rand::thread_rng().gen::<f64>() * total;

    let mut idx = 0usize;
    for (i, w) in weights.iter().enumerate() {
        r -= *w;
        if r <= 0.0 {
            idx = i;
            break;
        }
    }

    // 2️⃣ multiplier → payout (integer math, no rounding issues)
    let (num, den) = MUL[mi][idx];
    let payout = bet
        .checked_mul(num)
        .unwrap_or(i64::MAX)   // defensive – unlikely
        / den;

    (payout, idx)
}
