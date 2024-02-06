bindgen \
--impl-debug \
--impl-partialeq  \
--generate-inline-functions \
--with-derive-default \
--allowlist-var "^UL.*|JS.*|ul.*|WK.*" \
--allowlist-type "^UL.*|JS.*|ul.*|WK.*" \
--allowlist-function "^UL.*|JS.*|ul.*|WK.*" \
ultralight_api/AppCore/CAPI.h -o src/bindings.rs \
-- -Iultralight_api
