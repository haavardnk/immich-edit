use uuid::Uuid;

use crate::immich::dto::StackDetail;
use crate::immich::{ImmichClient, ImmichError};
use crate::state::AppState;

use super::StackPrimary;
use super::post_upload::Uploaded;

#[derive(Debug, PartialEq, Eq)]
struct StackPlan {
    ids: Vec<Uuid>,
    primary: Uuid,
}

pub(super) async fn stack_with_original(
    state: &AppState,
    immich: &ImmichClient,
    owner: Uuid,
    upload: &Uploaded<'_>,
) -> Result<(), ImmichError> {
    let source = upload.original.id.source();
    let _turn = state.stack_locks.lock((owner, source)).await;
    let original = immich.asset(source).await?;
    let stack = match original.current_stack_id() {
        Some(id) => Some(immich.get_stack(id).await?),
        None => None,
    };
    let plan = plan_stack(
        upload.id,
        source,
        stack.as_ref(),
        upload.body.stack_primary,
        upload.variant,
    );
    let created = immich.create_stack(&plan.ids).await?;
    if created.primary_asset_id != plan.primary {
        immich
            .update_stack_primary(created.id, plan.primary)
            .await?;
    }
    Ok(())
}

fn plan_stack(
    new_id: Uuid,
    source: Uuid,
    stack: Option<&StackDetail>,
    primary: StackPrimary,
    variant: i64,
) -> StackPlan {
    let primary = match (primary, stack) {
        (StackPrimary::Original, _) => source,
        (StackPrimary::Edited, Some(stack)) if variant > 0 => stack.primary_asset_id,
        (StackPrimary::Edited, _) => new_id,
    };
    let members = stack
        .into_iter()
        .flat_map(|stack| stack.assets.iter().map(|asset| asset.id.source()));
    let mut ids = vec![primary];
    for id in [new_id, source].into_iter().chain(members) {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    StackPlan { ids, primary }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset_key::AssetKey;
    use crate::immich::dto::AssetSummary;

    fn stack(primary: Uuid, members: &[Uuid]) -> StackDetail {
        let assets = members
            .iter()
            .map(|id| {
                serde_json::from_value::<AssetSummary>(serde_json::json!({
                    "id": AssetKey::master(*id).to_string()
                }))
                .unwrap()
            })
            .collect();
        StackDetail {
            id: Uuid::new_v4(),
            primary_asset_id: primary,
            assets,
        }
    }

    #[test]
    fn a_first_upload_starts_a_stack_on_top() {
        let (new_id, source) = (Uuid::new_v4(), Uuid::new_v4());

        for variant in [0, 1] {
            assert_eq!(
                plan_stack(new_id, source, None, StackPrimary::Edited, variant),
                StackPlan {
                    ids: vec![new_id, source],
                    primary: new_id
                }
            );
        }
        assert_eq!(
            plan_stack(new_id, source, None, StackPrimary::Original, 0),
            StackPlan {
                ids: vec![source, new_id],
                primary: source
            }
        );
    }

    #[test]
    fn a_later_upload_takes_in_every_member_of_the_stack() {
        let (first, second, source) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let existing = stack(first, &[first, source]);

        let plan = plan_stack(second, source, Some(&existing), StackPrimary::Edited, 0);

        assert_eq!(plan.primary, second);
        assert_eq!(plan.ids, [second, source, first]);
    }

    #[test]
    fn a_later_variant_keeps_the_primary_of_an_existing_stack() {
        let (first, second, source) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let existing = stack(first, &[first, source]);

        let plan = plan_stack(second, source, Some(&existing), StackPrimary::Edited, 1);

        assert_eq!(plan.primary, first);
        assert_eq!(plan.ids, [first, second, source]);
    }

    #[test]
    fn the_primary_joins_even_when_the_stack_lists_no_members() {
        let (first, second, source) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let existing = stack(first, &[]);

        let plan = plan_stack(second, source, Some(&existing), StackPrimary::Edited, 2);

        assert_eq!(plan.ids, [first, second, source]);
    }
}
