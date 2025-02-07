[1mdiff --git a/components/content/src/page.rs b/components/content/src/page.rs[m
[1mindex 9fff5ed4..71fed715 100644[m
[1m--- a/components/content/src/page.rs[m
[1m+++ b/components/content/src/page.rs[m
[36m@@ -198,24 +198,18 @@[m [mimpl Page {[m
             let parent_dir = path.parent().unwrap();[m
             page.assets = find_related_assets(parent_dir, config, true);[m
             if !page.assets.is_empty() {[m
[32m+[m[32m                let colocated_path = page[m
[32m+[m[32m                    .file[m
[32m+[m[32m                    .colocated_path[m
[32m+[m[32m                    .as_ref()[m
[32m+[m[32m                    .expect("Should have colocated path for assets");[m
                 page.serialized_assets = serialize_assets([m
                     &page.assets,[m
                     page.file.path.parent().unwrap(),[m
[31m-                    page.file[m
[31m-                        .colocated_path[m
[31m-                        .as_ref()[m
[31m-                        .expect("Should have colocated path for assets"),[m
[31m-                );[m
[31m-            }[m
[31m-            if !page.serialized_assets.is_empty() {[m
[31m-                page.assets_permalinks = get_assets_permalinks([m
[31m-                    &page.serialized_assets,[m
[31m-                    &page.permalink,[m
[31m-                    page.file[m
[31m-                        .colocated_path[m
[31m-                        .as_ref()[m
[31m-                        .expect("Should have colocated path for assets"),[m
[32m+[m[32m                    colocated_path,[m
                 );[m
[32m+[m[32m                page.assets_permalinks =[m
[32m+[m[32m                    get_assets_permalinks(&page.serialized_assets, &page.permalink, colocated_path);[m
             }[m
         } else {[m
             page.assets = vec![];[m
[1mdiff --git a/components/content/src/section.rs b/components/content/src/section.rs[m
[1mindex 2cf36644..b4ac035e 100644[m
[1m--- a/components/content/src/section.rs[m
[1m+++ b/components/content/src/section.rs[m
[36m@@ -130,25 +130,20 @@[m [mimpl Section {[m
         let parent_dir = path.parent().unwrap();[m
         section.assets = find_related_assets(parent_dir, config, false);[m
         if !section.assets.is_empty() {[m
[32m+[m[32m            let colocated_path = section[m
[32m+[m[32m                .file[m
[32m+[m[32m                .colocated_path[m
[32m+[m[32m                .as_ref()[m
[32m+[m[32m                .expect("Should have colocated path for assets");[m
             section.serialized_assets = serialize_assets([m
                 &section.assets,[m
                 section.file.path.parent().unwrap(),[m
[31m-                section[m
[31m-                    .file[m
[31m-                    .colocated_path[m
[31m-                    .as_ref()[m
[31m-                    .expect("Should have colocated path for assets"),[m
[32m+[m[32m                colocated_path,[m
             );[m
[31m-        }[m
[31m-        if !section.serialized_assets.is_empty() {[m
             section.assets_permalinks = get_assets_permalinks([m
                 &section.serialized_assets,[m
                 &section.permalink,[m
[31m-                section[m
[31m-                    .file[m
[31m-                    .colocated_path[m
[31m-                    .as_ref()[m
[31m-                    .expect("Should have colocated path for assets"),[m
[32m+[m[32m                colocated_path,[m
             );[m
         }[m
 [m
