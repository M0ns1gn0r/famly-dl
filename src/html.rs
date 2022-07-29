use chrono::Datelike;

use crate::child_info::ChildInfo;
use crate::post::Post;

pub fn render_post(post: &Post, child: &ChildInfo) -> String {
    let mut photos = String::new();
    for p in &post.photos {
        let tagged_style = if p.is_tagged(&child.id) { "border: 1px violet solid;" } else { "" };
        let img = format!(r#"<a target="_blank" href="photos/{file_name}" style="text-decoration: none;">
    <img src="photos/{file_name}" class="img-thumbnail mb-1" style="max-height: 240px;{tagged_style}" />
</a>"#,
            file_name = p.get_file_name(),
            tagged_style = tagged_style);

        photos.push_str(img.as_str());
    }

    let mut comments = String::new();
    if !post.comments.is_empty() {
        comments.push_str(r#"<hr /><h4 class="mb-3">Comments:</h4>"#);
        for c in &post.comments {
            let comment = format!(r#"<div class="bg-light border p-2 mb-2 rounded-3">
💬<b class="ms-1">{0}</b>
<br>
<div style="white-space: pre-line;">{1}</div>
</div>
"#, &c.author, &c.text);
            comments.push_str(comment.as_str());
        }
    }

    format!(
        r#"<!doctype html>
<html>
<head>
    <meta charset="utf-8">
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.2.0/dist/css/bootstrap.min.css" rel="stylesheet"
        integrity="sha384-gH2yIJqKdNHPEq0n4Mqa/HGKIhSkIHeL5AyhkYV8i59U5AR6csBvApHHNl/vI1Bx" crossorigin="anonymous">
</head>
<body class="container py-3" style="max-width: 1000px;">
    <p>
        <b>{author}</b>
        <br>
        {date}
    </p>
    <hr />
    <div style="white-space: pre-line;">{text}</div>
    <br />
    <div>{photos}</div>
    <div>{comments}</div>
</body>
</html>"#,
        author = post.author,
        date = post.date.with_timezone(&chrono::Local).to_rfc2822(),
        text = post.text,
        comments = comments)
}

pub fn render_index(posts: &Vec<Post>, has_tagged_photos: bool) -> String {
    let mut posts_html = String::new();
    if posts.len() > 0 {
        posts_html.push_str(r#"
    <h3>Posts</h3>
    <table class="table">
        <thead>
            <tr>
                <th scope="col">Year</th>
                <th scope="col">Month</th>
                <th scope="col">Link</th>
            </tr>
        </thead>
        <tbody>"#);

        let mut prev_month = 0;
        for p in posts {
            let month = p.date.month();
            if prev_month != month {
                // Next month.

                if prev_month != 0 {
                    // Close previous.
                    posts_html.push_str("</th></tr>");
                }
                // Open next.
                posts_html.push_str(format!(r#"
<tr>
    <th scope="col">{}</th>
    <th scope="col">{:02}</th>
    <th scope="col">"#, p.date.year(), p.date.month()).as_str());

                prev_month = month;
            }

            posts_html.push_str(format!(
                r#"<a href="posts/{}">{}</a><br />"#,
                p.get_file_name(),
                p.get_title(false)).as_str());
        }

        // Close table.
        posts_html.push_str(r#"
        </tbody>
    </table>"#);
    }

    let tagged_photos_html = if has_tagged_photos {
        r#"<h3>Tagged photos</h3>
    <a href="tagged_photos">Click to open</a>"#
    } else {
        ""
    };

    format!(
        r#"<!doctype html>
<html>
<head>
    <meta charset="utf-8">
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.2.0/dist/css/bootstrap.min.css" rel="stylesheet"
        integrity="sha384-gH2yIJqKdNHPEq0n4Mqa/HGKIhSkIHeL5AyhkYV8i59U5AR6csBvApHHNl/vI1Bx" crossorigin="anonymous">
</head>
<body class="container py-3" style="max-width: 1000px;">
    {posts}
    {tagged_photos}
</body>
</html>"#,
        posts = posts_html,
        tagged_photos = tagged_photos_html)
}