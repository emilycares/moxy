use crate::configuration::{get_route, Route};

#[test]
fn static_route() {
    let routes = vec![Route {
        method: "get".to_string(),
        path: "/api/test".to_string(),
        resource: "db/api/test.json".to_string(),
    }];
    let result = get_route(&routes, "http://localhost:8080/api/test").unwrap();

    assert_eq!(result.resource, routes[0].resource);
}

#[test]
fn dynamic_route_with_different_start() {
    let routes = vec![
        Route {
            method: "get".to_string(),
            path: "/api/test/1/$$$.json".to_string(),
            resource: "db/api/1/$$$.json".to_string(),
        },
        Route {
            method: "get".to_string(),
            path: "/api/test/2/$$$.json".to_string(),
            resource: "db/api/2/$$$.json".to_string(),
        },
        Route {
            method: "get".to_string(),
            path: "/api/test/3/$$$.json".to_string(),
            resource: "db/api/3/$$$.json".to_string(),
        },
    ];

    assert_eq!(
        get_route(&routes, "http://localhost:8080/api/test/1/abc.json")
            .unwrap()
            .resource,
        "db/api/1/$$$.json"
    );
    assert_eq!(
        get_route(&routes, "http://localhost:8080/api/test/2/abc.json")
            .unwrap()
            .resource,
        "db/api/2/$$$.json"
    );
    assert_eq!(
        get_route(&routes, "http://localhost:8080/api/test/3/abc.json")
            .unwrap()
            .resource,
        "db/api/3/$$$.json"
    );
}

#[test]
fn dynamic_route_with_different_end() {
    let routes = vec![
        Route {
            method: "get".to_string(),
            path: "/api/test/$$$.txt".to_string(),
            resource: "db/api/$$$.txt".to_string(),
        },
        Route {
            method: "get".to_string(),
            path: "/api/test/$$$.json".to_string(),
            resource: "db/api/$$$.json".to_string(),
        },
    ];

    assert_eq!(
        get_route(&routes, "http://localhost:8080/api/test/abc.txt")
            .unwrap()
            .resource,
        "db/api/$$$.txt"
    );
    assert_eq!(
        get_route(&routes, "http://localhost:8080/api/test/abc.json")
            .unwrap()
            .resource,
        "db/api/$$$.json"
    );
}
