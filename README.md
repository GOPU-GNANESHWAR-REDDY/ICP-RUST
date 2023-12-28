# Social Group Management Canister

This Rust Canister code provides functionality for managing social groups of developers on the Internet Computer platform. Developers can create profiles, join social groups based on shared ideas, and communicate within those groups.

## Features

- **Developer Profiles:** Developers can create profiles with their name, location, and a list of ideas they are interested in.

- **Social Groups:** Developers can create social groups, each associated with a unique idea. Members of a group share the same idea.

- **Joining Groups:** Developers can join social groups based on their shared ideas. To join a group, a developer's ideas must match the group's idea.

- **Group Communication:** Once in a group, developers can send messages to the entire group. Messages are associated with specific groups.

## Usage

### Creating Developer Profiles

To create a developer profile, use the `create_developer_profile` function with the developer's name, location, and a list of ideas:

```rust
create_developer_profile("John Doe", "City A", vec!["Web Development", "AI"]);

//rust
create_social_group("Web Developers Group", "Web Development");

//rust
join_social_group(developer_id, group_id);

//rust
send_message(sender_id, group_id, "Hello, everyone! Let's discuss our projects.".to_string());


git clone https://github.com/your/repository.git
cd your-repository

dfx build

dfx canister --network ic create social_groups
dfx canister --network ic install social_groups

