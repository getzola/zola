function formatSearchResultHeader(term, count) {
  if (count === 0) {
    return "No search results for '" + term + "'.";
  }

  return count + " search result" + count > 1 ? "s" : "" + " for '" + term + "':";
}

function formatSearchResultItem(term, item) {
  console.log(item);
  return '<div class="search-results__item">'
  + item
  + '</div>';
}

function initSearch() {
  var $searchInput = document.getElementById("search");
  var $searchResults = document.querySelector(".search-results");
  var $searchResultsHeader = document.querySelector(".search-results__headers");
  var $searchResultsItems = document.querySelector(".search-results__items");

  var options = {
    bool: "AND",
    expand: true,
    teaser_word_count: 30,
    limit_results: 30,
    fields: {
      title: {boost: 2},
      body: {boost: 1},
    }
  };
  var currentTerm = "";
  var index = elasticlunr.Index.load(window.searchIndex);

  $searchInput.addEventListener("keyup", function() {
    var term = $searchInput.value.trim();
    if (!index || term === "" || term === currentTerm) {
      return;
    }
    $searchResults.style.display = term === "" ? "block" : "none";
    $searchResultsItems.innerHTML = "";
    var results = index.search(term, options);
    currentTerm = term;
    $searchResultsHeader.textContent = searchResultText(term, results.length);
    for (var i = 0; i < results.length; i++) {
      var item = document.createElement("li");
      item.innerHTML = formatSearchResult(results[i], term);
      $searchResultsItems.appendChild(item);
    }
  });
}


if (document.readyState === "complete" ||
    (document.readyState !== "loading" && !document.documentElement.doScroll)
) {
  initSearch();
} else {
  document.addEventListener("DOMContentLoaded", initSearch);
}
