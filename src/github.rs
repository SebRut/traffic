use tokio_core;
use views::*;

use reqwest::unstable::async::{Client, Response};
use futures::Future;
use futures::future::join_all;

#[derive(Deserialize, Debug)]
pub struct Repository {
    pub full_name: String,
    pub name: String
}

#[derive(Debug)]
pub struct RepoDetails {
    pub repository: Repository,
    pub views: CountsForTwoWeeks,
    pub clones: CountsForTwoWeeks
}

pub fn get_all_traffic_data(username: &str, password: &str) -> Vec<RepoDetails> {
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let client = Client::new(&core.handle());

    let repos =
        core.run(
        client
            .get("https://api.github.com/user/repos?sort=updated&affiliation=owner")
            .basic_auth(username, Some(password.clone()))
            .send()
            .and_then(|mut res : Response| {
                res.json::<Vec<Repository>>()
            })
        ).unwrap();

    let mut traffic_requests = vec![];
    let mut clones_requests = vec![];

    for repo in &repos {
       let request = client
            .get(&format!("https://api.github.com/repos/{}/traffic/views", repo.full_name))
            .basic_auth(username, Some(password.clone()))
            .send()
            .and_then(|mut res : Response| {
                res.json::<ViewsForTwoWeeks>()
            });
        traffic_requests.push(request);
                    let clones_request = client
                .get(&format!("https://api.github.com/repos/{}/traffic/clones", repo.full_name))
                .basic_auth(username, Some(password.clone()))
                .send()
                .and_then(|mut res : Response| {
                    res.json::<ClonesForTwoWeeks>()
                });
            clones_requests.push(clones_request);
    }

    let work = join_all(traffic_requests);
    let clones_work = join_all(clones_requests);

    let mut repo_details : Vec<RepoDetails> = vec![];

    let all_views = core.run(work).unwrap();
    let all_clones = core.run(clones_work).unwrap();

    for (views, clones, repo) in all_views.into_iter().zip(all_clones.into_iter()).zip(repos.into_iter()).map(|((v,c),r)| (v,c,r)) {
        let views = CountsForTwoWeeks::from(views);
        let clones = CountsForTwoWeeks::from(clones);
        repo_details.push(RepoDetails { repository: repo, views, clones });
    }

    repo_details
}
