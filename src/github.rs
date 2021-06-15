use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GitHubSHA(String);

#[derive(Deserialize, Debug)]
pub struct GitHubRef(String);

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
    archive_url: GitHubURL,
    archived: bool,
    assignees_url: GitHubURL,
    blobs_url: GitHubURL,
    branches_url: GitHubURL,
    clone_url: GitHubURL,
    collaborators_url: GitHubURL,
    comments_url: GitHubURL,
    commits_url: GitHubURL,
    compare_url: GitHubURL,
    contents_url: GitHubURL,
    contributors_url: GitHubURL,
    created_at: u64,
    default_branch: String,
    deployments_url: GitHubURL,
    description: String,
    disabled: bool,
    downloads_url: GitHubURL,
    events_url: GitHubURL,
    fork: bool,
    forks: usize,
    forks_count: usize,
    forks_url: GitHubURL,
    full_name: String,
    git_commits_url: GitHubURL,
    git_refs_url: GitHubURL,
    git_tags_url: GitHubURL,
    git_url: GitHubURL,
    has_downloads: bool,
    has_issues: bool,
    has_pages: bool,
    has_projects: bool,
    has_wiki: bool,
    homepage: Option<GitHubURL>,
    hooks_url: GitHubURL,
    html_url: GitHubURL,
    id: u64,
    issue_comment_url: GitHubURL,
    issue_events_url: GitHubURL,
    issues_url: GitHubURL,
    keys_url: GitHubURL,
    labels_url: GitHubURL,
    language: String,
    languages_url: GitHubURL,
    license: Option<String>,
    master_branch: String,
    merges_url: GitHubURL,
    milestones_url: GitHubURL,
    mirror_url: Option<GitHubURL>,
    name: String,
    node_id: String,
    notifications_url: GitHubURL,
    open_issues: usize,
    open_issues_count: usize,
    owner: GitHubUser,
    private: bool,
    pulls_url: GitHubURL,
    pushed_at: u64,
    releases_url: GitHubURL,
    size: usize,
    ssh_url: GitHubURL,
    stargazers: usize,
    stargazers_count: usize,
    stargazers_url: GitHubURL,
    statuses_url: GitHubURL,
    subscribers_url: GitHubURL,
    subscription_url: GitHubURL,
    svn_url: GitHubURL,
    tags_url: GitHubURL,
    teams_url: GitHubURL,
    trees_url: GitHubURL,
    updated_at: String,
    url: GitHubURL,
    watchers: usize,
    watchers_count: usize,
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
    after: Option<GitHubSHA>,
    before: Option<GitHubSHA>,

    base_ref: Option<GitHubRef>,
    commits: Option<Vec<GitHubCommit>>,

    compare: Option<GitHubURL>,

    created: bool,
    deleted: bool,
    forced: bool,

    head_commit: GitHubCommit,
    pusher: GitUser,

    #[serde(rename = "ref")]
    reference: GitHubRef,

    repository: GitHubRepository,
    sender: GitHubUser,
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
