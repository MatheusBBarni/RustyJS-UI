#[derive(Clone, Copy, Debug)]
pub struct ScriptAsset {
    pub path: &'static str,
    pub source: &'static str,
}

const BOOTSTRAP: ScriptAsset = ScriptAsset {
    path: concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/runtime/jsengine/bootstrap.js"
    ),
    source: include_str!("bootstrap.js"),
};

const COUNTER_APP: ScriptAsset = ScriptAsset {
    path: concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/runtime/jsengine/counter_app.js"
    ),
    source: include_str!("counter_app.js"),
};

pub const fn bootstrap() -> ScriptAsset {
    BOOTSTRAP
}

pub const fn counter_app() -> ScriptAsset {
    COUNTER_APP
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_uses_string_callback_ids() {
        let source = bootstrap().source;
        assert!(source.contains("return id;"));
        assert!(!source.contains("return { id };"));
    }

    #[test]
    fn bootstrap_defends_pending_state_on_render_errors() {
        let source = bootstrap().source;
        assert!(source.contains("finally {"));
        assert!(source.contains("this.isRenderPending = false;"));
    }

    #[test]
    fn bootstrap_exposes_flat_list_helper() {
        let source = bootstrap().source;
        assert!(source.contains("function createFlatList(props = {})"));
        assert!(source.contains("globalThis.FlatList = (props) => createFlatList(props);"));
        assert!(source.contains("return createNode('FlatList', {"));
    }

    #[test]
    fn bootstrap_exposes_fetch_bridge_contract() {
        let source = bootstrap().source;
        assert!(source.contains("globalThis.fetch = (url, options = {}) => new Promise("));
        assert!(source.contains("action: 'FETCH_REQUEST'"));
        assert!(source.contains("requestId"));
        assert!(source.contains("resolveFetch"));
        assert!(source.contains("rejectFetch"));
        assert!(source.contains("method"));
        assert!(source.contains("headers"));
        assert!(source.contains("body"));
    }

    #[test]
    fn bootstrap_exposes_router_helper() {
        let source = bootstrap().source;
        assert!(source.contains("class AppRouter"));
        assert!(source.contains("createRouter(config = {})"));
        assert!(source.contains("getPath()"));
    }

    #[test]
    fn bundled_sample_app_matches_prd_shape() {
        let source = counter_app().source;
        assert!(source.contains("function AppLayout()"));
        assert!(source.contains("App.requestRender();"));
        assert!(source.contains("Button({"));
        assert!(source.contains("Text({"));
    }
}
