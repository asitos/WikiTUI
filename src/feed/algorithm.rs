use crate::feed::profile::FeedProfile;

#[derive(Debug, Clone)]
pub struct FeedItem {
    pub title: String,
    pub short_description: Option<String>,
    pub snippet: String,
    pub categories: Vec<String>,
    pub is_liked: bool,
}

pub enum SelectionStrategy {
    WeightedCategory,
    TopCategory,
    RandomExploration,
}

pub fn rand_u64() -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(1234567) as u64;
    let mut x = nanos ^ 0x517cc1b727220a95;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    x.wrapping_mul(0x2545f4914f6cdd1d)
}

pub fn shuffle<T>(slice: &mut [T]) {
    let mut rng = rand_u64();
    for i in (1..slice.len()).rev() {
        rng ^= rng >> 12;
        rng ^= rng << 25;
        rng ^= rng >> 27;
        let j = (rng.wrapping_mul(0x2545f4914f6cdd1d) as usize) % (i + 1);
        slice.swap(i, j);
    }
}

pub fn choose_strategy() -> SelectionStrategy {
    let roll = (rand_u64() % 100) as u8;
    if roll < 40 {
        SelectionStrategy::WeightedCategory
    } else if roll < 82 {
        SelectionStrategy::TopCategory
    } else {
        SelectionStrategy::RandomExploration
    }
}

pub fn select_best_item(mut candidates: Vec<FeedItem>, profile: &FeedProfile) -> Option<FeedItem> {
    if candidates.is_empty() {
        return None;
    }

    let strategy = choose_strategy();
    match strategy {
        SelectionStrategy::RandomExploration => {
            let idx = (rand_u64() as usize) % candidates.len();
            Some(candidates.swap_remove(idx))
        }
        SelectionStrategy::TopCategory | SelectionStrategy::WeightedCategory => {
            let mut best_idx = None;
            let mut best_score = i32::MIN;

            for (idx, item) in candidates.iter().enumerate() {
                if profile.seen_articles.contains(&item.title) {
                    continue;
                }
                let score = profile.score_for_categories(&item.categories);
                if score > best_score {
                    best_score = score;
                    best_idx = Some(idx);
                }
            }

            if let Some(idx) = best_idx {
                Some(candidates.swap_remove(idx))
            } else {
                candidates.into_iter().next()
            }
        }
    }
}
