use lopdf::{Bookmark, Document, Object, ObjectId};
use std::{collections::BTreeMap, io::Result, path::Path};

/// Merge PDF files together, in the order they are being provided.
///
/// This function comes almost straight from the "Merge PDF documents" example
/// from the `lopdf` README.md:
/// https://github.com/J-F-Liu/lopdf/blob/850b150461245cbf7c8dd780b31c76837769a0f5/README.md
pub fn combine_documents(documents: Vec<Document>) -> Result<Document> {
    // Define a starting max_id (will be used as start index for object_ids)
    let mut max_id = 1;
    let mut pagenum = 1;

    // Collect all Documents Objects grouped by a map
    let mut documents_pages = BTreeMap::new();
    let mut documents_objects = BTreeMap::new();
    let mut document = Document::with_version("1.5");

    for mut doc in documents {
        let mut first = false;
        doc.renumber_objects_with(max_id);

        max_id = doc.max_id + 1;

        documents_pages.extend(
            doc.get_pages()
                .into_values()
                .map(|object_id| {
                    if !first {
                        let bookmark = Bookmark::new(
                            format!("Page_{}", pagenum),
                            [0.0, 0.0, 1.0],
                            0,
                            object_id,
                        );
                        document.add_bookmark(bookmark, None);
                        first = true;
                        pagenum += 1;
                    }

                    (object_id, doc.get_object(object_id).unwrap().to_owned())
                })
                .collect::<BTreeMap<ObjectId, Object>>(),
        );
        documents_objects.extend(doc.objects);
    }

    // Catalog and Pages are mandatory
    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    // Process all objects except "Page" type
    for (object_id, object) in documents_objects.iter() {
        // We have to ignore "Page" (as are processed later), "Outlines" and "Outline" objects
        // All other objects should be collected and inserted into the main Document
        match object.type_name().unwrap_or(b"") {
            b"Catalog" => {
                // Collect a first "Catalog" object and use it for the future "Pages"
                catalog_object = Some((
                    if let Some((id, _)) = catalog_object {
                        id
                    } else {
                        *object_id
                    },
                    object.clone(),
                ));
            }
            b"Pages" => {
                // Collect and update a first "Pages" object and use it for the future "Catalog"
                // We have also to merge all dictionaries of the old and the new "Pages" object
                if let Ok(dictionary) = object.as_dict() {
                    let mut dictionary = dictionary.clone();
                    if let Some((_, ref object)) = pages_object {
                        if let Ok(old_dictionary) = object.as_dict() {
                            dictionary.extend(old_dictionary);
                        }
                    }

                    pages_object = Some((
                        if let Some((id, _)) = pages_object {
                            id
                        } else {
                            *object_id
                        },
                        Object::Dictionary(dictionary),
                    ));
                }
            }
            b"Page" => {}     // Ignored, processed later and separately
            b"Outlines" => {} // Ignored, not supported yet
            b"Outline" => {}  // Ignored, not supported yet
            _ => {
                document.objects.insert(*object_id, object.clone());
            }
        }
    }

    // If no "Pages" found abort
    if pages_object.is_none() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Pages root not found.",
        ));
    }

    // Iter over all "Page" and collect with the parent "Pages" created before
    for (object_id, object) in documents_pages.iter() {
        if let Ok(dictionary) = object.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Parent", pages_object.as_ref().unwrap().0);

            document
                .objects
                .insert(*object_id, Object::Dictionary(dictionary));
        }
    }

    // If no "Catalog" found abort
    if catalog_object.is_none() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Catalog root not found.",
        ));
    }

    let catalog_object = catalog_object.unwrap();
    let pages_object = pages_object.unwrap();

    // Build a new "Pages" with updated fields
    if let Ok(dictionary) = pages_object.1.as_dict() {
        let mut dictionary = dictionary.clone();

        // Set new pages count
        dictionary.set("Count", documents_pages.len() as u32);

        // Set new "Kids" list (collected from documents pages) for "Pages"
        dictionary.set(
            "Kids",
            documents_pages
                .into_keys()
                .map(|(object_id, other)| Object::Reference((object_id, other)))
                .collect::<Vec<_>>(),
        );
        document
            .objects
            .insert(pages_object.0, Object::Dictionary(dictionary));
    }

    // Build a new "Catalog" with updated fields
    if let Ok(dictionary) = catalog_object.1.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Pages", pages_object.0);
        dictionary.remove(b"Outlines"); // Outlines not supported in merged PDFs

        document
            .objects
            .insert(catalog_object.0, Object::Dictionary(dictionary));
    }
    document.trailer.set("Root", catalog_object.0);

    // Update the max internal ID as wasn't updated before due to direct objects insertion
    document.max_id = document.objects.len() as u32;

    // Reorder all new Document objects
    document.renumber_objects();

    // Set any Bookmarks to the First child if they are not set to a page
    document.adjust_zero_pages();

    // Set all bookmarks to the PDF Object tree then set the Outlines to the Bookmark content map.
    if let Some(n) = document.build_outline() {
        if let Ok(Object::Dictionary(ref mut dict)) = document.get_object_mut(catalog_object.0) {
            dict.set("Outlines", Object::Reference(n));
        }
    }
    document.compress();

    Ok(document)
}

pub fn combine_mem(documents: &[&[u8]]) -> Result<Document> {
    // Load the buffers.
    let docs = documents
        .iter()
        .map(|buffer| Document::load_mem(buffer).unwrap())
        .collect::<Vec<Document>>();

    combine_documents(docs)
}

pub fn combine_pdf<P>(documents: &[&Path], output: P) -> Result<()>
where
    P: AsRef<Path>,
{
    // Load the files.
    let docs = documents
        .iter()
        .map(|d| Document::load(d).unwrap())
        .collect::<Vec<Document>>();

    // Merge the files.
    let mut combined = combine_documents(docs)?;

    // Save the merged PDF.
    combined.save(output)?;
    Ok(())
}

/// Append the `extra` document to all the `documents`.
pub fn batch_append(documents: &[&Path], extra: &Path) -> Result<()> {
    for &document in documents {
        combine_pdf(&[document, extra], document)?;
    }

    Ok(())
}
