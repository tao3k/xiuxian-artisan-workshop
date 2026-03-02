/// Declarative helper to define native tools with minimal boilerplate.
macro_rules! define_native_tool {
    (
        $(#[$meta:meta])*
        $vis:vis struct $tool:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field:ident : $field_ty:ty
            ),* $(,)?
        }
        name: $name:expr,
        description: $description:expr,
        parameters: $parameters:expr,
        call(|$self_ident:ident, $arguments_ident:ident, $context_ident:ident| $body:block)
    ) => {
        $(#[$meta])*
        $vis struct $tool {
            $(
                $(#[$field_meta])*
                $field_vis $field: $field_ty,
            )*
        }

        #[async_trait::async_trait]
        impl super::registry::NativeTool for $tool {
            fn name(&self) -> &str {
                $name
            }

            fn description(&self) -> &str {
                $description
            }

            fn parameters(&self) -> serde_json::Value {
                $parameters
            }

            async fn call(
                &self,
                $arguments_ident: Option<serde_json::Value>,
                $context_ident: &super::registry::NativeToolCallContext,
            ) -> anyhow::Result<String> {
                let $self_ident = self;
                $body
            }
        }
    };
}

pub(crate) use define_native_tool;
