use skyscraper::html::HtmlNode;
use skyscraper::{html, xpath};

#[test]
fn txpath() {
    let html_text = r##"
<html>
    <body>
        <div >Hello<h1> xx</h1> world</div>
    </body>
</html>"##;

    let document = html::parse(html_text).unwrap();
    let document = foo(document);

    let expr = xpath::parse("//div").unwrap();

    for x in expr.apply(&document).unwrap() {
        let x = foo(x);

        println!("{}", x.get_text(&document).unwrap());
        let node = document.get_html_node(&x).unwrap();

        match node {
            HtmlNode::Tag(x) => {
                println!("tag {:?}", x.attributes);
            }
            HtmlNode::Text(x) => {
                println!("text = {}", x);
            }
        }
    }
}

fn foo<T: Send + Sync>(x: T) -> T {
    x
}
