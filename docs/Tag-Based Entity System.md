# Tag-Based Entity System (TBES)

TBES is an in-progress organization system concept basing the entity placement, discovery and permission management not on any hierarchical structure (e.g. folders within folders, users within groups) but on the entity having a set of tags which identify its properties and actions it is allowed to take.

Main actions of the system:

- The user can query for all entities that have the specified set of tags, and retrieve a paged list of Identities matching the query.
- The user can query for details of one entity by its Identity, and retrieve all tags it has been assigned.
- The user can create new tags
- The user can assign tags to entities
- The user can unassign tags from entities

### General types of tags:

- Boolean tags - The tag is either assigned to the entity or not. Optimization structure: Bitmap index
- Numeric tags - The tag is either not assigned to the entity, or assigned with a specific 64bit value. Optimization structure: B+-tree. This tag can be assigned multiple times to the same entity, but with different values.

### Specific subtypes of tags:

- Timestamp tags - Alias for numeric tags with the value being formatted as a timestamp
- Tag tag - Assigned only to entities which are tags themselves
- Identity read tag - Assigned to entities which can be read by the Identity holder
- Identity write tag - Assigned to entities which can be written to by the Identity holder
- Identity execute tag - Assigned to entities which can be executed by the Identity holder
- Text tags - By this we mean a tag with a text value payload. They are realized by actually being separate tags. Identity tags are examples of Text tags

### Identity maintenance:

The Identity is basically an identifier uniquely assigned to one entity.

The Identity can be reassigned after the entity is removed, and is cleaned up from all indexes.

The Identity is a 64bit number, where the most significant 32 bits are the source ID (device/driver/group), and the least significant 32 bits are the ID of the entity in the respective source system.

An example would be a file in an HDD, with an Identity of `0000-003F-0000-1AFB`. In this case `0000-003F` is the drive identifier, and the `0000-1AFB` part is the identifier of the file within the drive.

### Querying capabilities:

Let's assume we have an `ls`-equivalent program available for TBES, called `ts`.

While normal `ls` works by listing all files belonging to a specific hierarchical path, `ts` lists all entities belonging to a specific set of tags.

##### Comparison:

`ls /usr/Micha_i/Music/RickAstley/`

`ts user:Micha_i,category:Music,author:Rick Astley`

Or, if we go with text tags:

`ts usr:Micha_i,music:Rick Astley`

Worth noticing is the fact that in mapping from a hierarchical to a tag-based system, all combinations of folders are treated as equal, e.g.:

1. `/usr/Micha_i/Music/RickAstley/`
2. `/usr/Music/Micha_i/RickAstley/`
3. `/usr/RickAstley/Music/Micha_i/`

The advantage of a tag system here is that it's very easy to change the search criteria.
For example, you may decide that you want to instead find all your music mp3 files which were not made by Rick Astley:

`ts user:Micha_i,category:Music,!author:Rick Astley,format:mp3`

### Considerations:

- Writing out the tags by hand takes much more time and line length than the equivalent path hierarchy
- The entity system will automatically add the user identity read tag to the query, so we could skip writing the user tag explicitly
- When saving or creating a file, the UX of tags would need to be sketched out, as that is a critical user action.
