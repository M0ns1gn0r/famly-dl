* ✅ Check the *target folder* exists or can be created
* ✅ Get the list of children `https://app.famly.de/api/v2/calendar/list`
* ✅ Ask to choose the target child and get its id
* Load a list of posts `https://app.famly.de/api/feed/feed/feed?olderThan=2022-07-22T10%3A49%3A48%2B00%3A00`

    ### ✅ Filter
    * Doesn't have `systemPostTypeClass: "Daycare.Checkin:CheckedIn"`
    * Has non-empty `body` (looks like empty ones are mostly invitations to events)
    ~~* Post is "liked" (will generate false positives for other children, but that's OK?)~~
    * Has at least one photo tagged with the target `childId`

    ### Then
    *  For each post create `child_first_name/year/month/<first25symbolsFromBody>...[2].htm` with sender name, text body, photos, ~~names of who gave a like~~, comments
    * Download each post's photo to `child_first_name/year/month/photos`
    * Create a hardlink to each **tagged** photo in the folder `child_first_name/tagged_photos`

* **LATER** Load a list of private messages `TODO LINK`
    
    ### Filter
    * Has at least one photo tagged with the target `childId`
    * ?

    ### Then
    * ?


* Load a list of all tagged photos `https://app.famly.de/api/v2/images/tagged?childId=<GUID>limit=100&olderThan=2022-02-22T13%3A44%3A49%2B00%3A00`
    * Ensure each of them was already downloaded to `child_first_name/tagged_photos`, if not do that.


* Create `child_first_name/index.htm` with such structure:
    ```
    POSTS
    YEAR.MONTH.DAY | <first25symbolsFromBody>...[2].htm (with a link to it)

    PRIVATE MESSAGES
    YEAR.MONTH.DAY | <first25symbolsFromBody>...[2].htm (with a link to it)

    TAGGED PHOTOS
    Link
    ```