#[macro_export]
macro_rules! impl_app {
      ($($body:tt)*) => {
          impl<IS, CS, C, Sub> $crate::domain::app::App<IS, CS, C, Sub>
          where
              IS: $crate::ports::services::image_store::ImageStore + Send + Sync,
              CS: $crate::ports::services::card_store::CardStore
                  + ::cards_sdk::SpoilerQueue + Send + Sync,
              C: $crate::ports::services::cache::Cache + Send + Sync,
              Sub: $crate::ports::services::spoiler_subscription::SpoilerSubscription
                  + Send + Sync,
          {
              $($body)*
          }
      };
  }

#[macro_export]
macro_rules! impl_async_for_app {
    ($trait:path { $($body:tt)* }) => {
          #[::async_trait::async_trait]
          impl<IS, CS, C, Sub> $trait for $crate::domain::app::App<IS, CS, C, Sub>
          where
              IS: $crate::ports::services::image_store::ImageStore + Send + Sync,
              CS: $crate::ports::services::card_store::CardStore
                  + ::cards_sdk::SpoilerQueue + Send + Sync,
              C: $crate::ports::services::cache::Cache + Send + Sync,
              Sub: $crate::ports::services::spoiler_subscription::SpoilerSubscription
                  + Send + Sync,
          {
              $($body)*
          }
    };
}
