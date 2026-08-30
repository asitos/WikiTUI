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

pub fn choose_strategy() -> SelectionStrategy {
    let roll: u8 = fastrand::u8(0..100);
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
            let idx = fastrand::usize(0..candidates.len());
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
