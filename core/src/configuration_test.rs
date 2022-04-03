use hyper::Uri;

use crate::configuration::{get_route, Route};

#[test]
fn static_route() {
    let routes = vec![Route {
        method: "get".to_string(),
        path: "/api/test".to_string(),
        resource: "db/api/test.json".to_string(),
    }];
    let url = &"http://localhost:8080/api/test".parse::<Uri>().unwrap();
    let (result, parameter) = get_route(&routes, &url, "get");

    assert_eq!(result.unwrap().resource, routes[0].resource);
    assert_eq!(parameter, None);
}

#[test]
fn dynamic_route_with_different_start() {
    let method = r#"get"#;

    let routes = vec![
        Route {
            method: method.to_string(),
            path: "/api/test/1/$$$.json".to_string(),
            resource: "db/api/1/$$$.json".to_string(),
        },
        Route {
            method: method.to_string(),
            path: "/api/test/2/$$$.json".to_string(),
            resource: "db/api/2/$$$.json".to_string(),
        },
        Route {
            method: method.to_string(),
            path: "/api/test/3/$$$.json".to_string(),
            resource: "db/api/3/$$$.json".to_string(),
        },
    ];

    assert_eq!(
        get_route(
            &routes,
            &"http://localhost:8080/api/test/1/abc.json"
                .parse::<Uri>()
                .unwrap(),
            method
        )
        .0
        .unwrap()
        .resource,
        "db/api/1/$$$.json"
    );
    assert_eq!(
        get_route(
            &routes,
            &"http://localhost:8080/api/test/2/abc.json"
                .parse::<Uri>()
                .unwrap(),
            method
        )
        .0
        .unwrap()
        .resource,
        "db/api/2/$$$.json"
    );
    assert_eq!(
        get_route(
            &routes,
            &"http://localhost:8080/api/test/3/abc.json"
                .parse::<Uri>()
                .unwrap(),
            method
        )
        .0
        .unwrap()
        .resource,
        "db/api/3/$$$.json"
    );
}

#[test]
fn dynamic_route_with_different_end() {
    let method = r#"get"#;
    let routes = vec![
        Route {
            method: method.to_string(),
            path: "/api/test/$$$.txt".to_string(),
            resource: "db/api/$$$.txt".to_string(),
        },
        Route {
            method: method.to_string(),
            path: "/api/test/$$$.json".to_string(),
            resource: "db/api/$$$.json".to_string(),
        },
    ];

    assert_eq!(
        get_route(
            &routes,
            &"http://localhost:8080/api/test/abc.txt"
                .parse::<Uri>()
                .unwrap(),
            method
        )
        .0
        .unwrap()
        .resource,
        "db/api/$$$.txt"
    );
    assert_eq!(
        get_route(
            &routes,
            &"http://localhost:8080/api/test/abc.json"
                .parse::<Uri>()
                .unwrap(),
            method
        )
        .0
        .unwrap()
        .resource,
        "db/api/$$$.json"
    );
}

#[test]
fn dynamic_paramerter_end() {
    let method = r#"get"#;
    let routes = vec![
        Route {
            method: method.to_string(),
            path: "/api/test/$$$".to_string(),
            resource: "db/api/$$$".to_string(),
        },
    ];

    assert_eq!(
        get_route(
            &routes,
            &"http://localhost:8080/api/test/abc"
                .parse::<Uri>()
                .unwrap(),
            method
        )
        .1
        .unwrap(),
        "abc"
    );
}

#[test]
fn dynamic_paramerter_middle() {
    let method = r#"get"#;
    let routes = vec![
        Route {
            method: method.to_string(),
            path: "/api/test/$$$.txt".to_string(),
            resource: "db/api/$$$.txt".to_string(),
        },
    ];

    assert_eq!(
        get_route(
            &routes,
            &"http://localhost:8080/api/test/abc.txt"
                .parse::<Uri>()
                .unwrap(),
            method
        )
        .1
        .unwrap(),
        "abc"
    );
}
