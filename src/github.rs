use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GitHubSHA(String);

#[derive(Deserialize, Debug)]
pub struct GitHubRef(pub String);

#[derive(Deserialize, Debug)]
pub struct GitHubURL(String);

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct GitUser {
    email: Option<String>,
    name: Option<String>,
    username: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct GitHubCommit {
    // Changes to files
    added: Vec<String>,
    modified: Vec<String>,
    removed: Vec<String>,

    id: GitHubSHA,
    author: GitUser,
    committer: GitUser,

    message: String,
    distinct: bool,
    timestamp: String,
    tree_id: GitHubSHA,
    url: String,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct GitHubRepository {
    pub archive_url: GitHubURL,
    pub archived: bool,
    pub assignees_url: GitHubURL,
    pub blobs_url: GitHubURL,
    pub branches_url: GitHubURL,
    pub clone_url: GitHubURL,
    pub collaborators_url: GitHubURL,
    pub comments_url: GitHubURL,
    pub commits_url: GitHubURL,
    pub compare_url: GitHubURL,
    pub contents_url: GitHubURL,
    pub contributors_url: GitHubURL,
    pub created_at: u64,
    pub default_branch: String,
    pub deployments_url: GitHubURL,
    pub description: String,
    pub disabled: bool,
    pub downloads_url: GitHubURL,
    pub events_url: GitHubURL,
    pub fork: bool,
    pub forks: usize,
    pub forks_count: usize,
    pub forks_url: GitHubURL,
    pub full_name: String,
    pub git_commits_url: GitHubURL,
    pub git_refs_url: GitHubURL,
    pub git_tags_url: GitHubURL,
    pub git_url: GitHubURL,
    pub has_downloads: bool,
    pub has_issues: bool,
    pub has_pages: bool,
    pub has_projects: bool,
    pub has_wiki: bool,
    pub homepage: Option<GitHubURL>,
    pub hooks_url: GitHubURL,
    pub html_url: GitHubURL,
    pub id: u64,
    pub issue_comment_url: GitHubURL,
    pub issue_events_url: GitHubURL,
    pub issues_url: GitHubURL,
    pub keys_url: GitHubURL,
    pub labels_url: GitHubURL,
    pub language: String,
    pub languages_url: GitHubURL,
    pub license: Option<String>,
    pub master_branch: String,
    pub merges_url: GitHubURL,
    pub milestones_url: GitHubURL,
    pub mirror_url: Option<GitHubURL>,
    pub name: String,
    pub node_id: String,
    pub notifications_url: GitHubURL,
    pub open_issues: usize,
    pub open_issues_count: usize,
    pub owner: GitHubUser,
    pub private: bool,
    pub pulls_url: GitHubURL,
    pub pushed_at: u64,
    pub releases_url: GitHubURL,
    pub size: usize,
    pub ssh_url: GitHubURL,
    pub stargazers: usize,
    pub stargazers_count: usize,
    pub stargazers_url: GitHubURL,
    pub statuses_url: GitHubURL,
    pub subscribers_url: GitHubURL,
    pub subscription_url: GitHubURL,
    pub svn_url: GitHubURL,
    pub tags_url: GitHubURL,
    pub teams_url: GitHubURL,
    pub trees_url: GitHubURL,
    pub updated_at: String,
    pub url: GitHubURL,
    pub watchers: usize,
    pub watchers_count: usize,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct GitHubUser {
    avatar_url: GitHubURL,
    email: Option<String>,
    events_url: GitHubURL,
    followers_url: GitHubURL,
    following_url: GitHubURL,
    gists_url: GitHubURL,
    gravatar_id: String,
    html_url: GitHubURL,
    id: u64,
    login: String,
    name: Option<String>,
    node_id: String,
    organizations_url: GitHubURL,
    received_events_url: GitHubURL,
    repos_url: GitHubURL,
    site_admin: bool,
    starred_url: GitHubURL,
    subscriptions_url: GitHubURL,
    #[serde(rename = "type")]
    user_type: String,
    url: GitHubURL,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]

pub struct GithubPushEvent {
    pub after: Option<GitHubSHA>,
    pub before: Option<GitHubSHA>,

    pub base_ref: Option<GitHubRef>,
    pub commits: Option<Vec<GitHubCommit>>,

    pub compare: Option<GitHubURL>,

    pub created: bool,
    pub deleted: bool,
    pub forced: bool,

    pub head_commit: GitHubCommit,
    pub pusher: GitUser,

    #[serde(rename = "ref")]
    pub reference: GitHubRef,

    pub repository: GitHubRepository,
    pub sender: GitHubUser,
}

#[cfg(test)]
pub mod test {

    use super::*;

    pub fn github_push_event_str() -> String {
        include_str!("../test_data/github_push_event.json").into()
    }

    pub fn github_commit_entry_str() -> String {
        include_str!("../test_data/github_commit_entry.json").into()
    }

    pub fn github_repository_entry_str() -> String {
        include_str!("../test_data/github_repository_entry.json").into()
    }

    pub fn github_user_entry_str() -> String {
        include_str!("../test_data/github_user_entry.json").into()
    }

    #[test]
    pub fn deserialize_github_push_event() {
        let _event: GithubPushEvent = serde_json::from_str(&github_push_event_str()).unwrap();
    }

    #[test]
    pub fn deserialize_github_commit_entry() {
        let _commit: GitHubCommit = serde_json::from_str(&&github_commit_entry_str()).unwrap();
    }

    #[test]
    pub fn deserialize_github_repository_entry() {
        let _commit: GitHubRepository =
            serde_json::from_str(&github_repository_entry_str()).unwrap();
    }

    #[test]
    pub fn deserialize_github_user_entry() {
        let _commit: GitHubUser = serde_json::from_str(&github_user_entry_str()).unwrap();
    }
}
