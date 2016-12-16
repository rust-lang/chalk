// ?F in_list: ?L
//
// Lists are actually trees and have the form:
//
// ```
// L<M> = M
//      | [L, L]
//      | Nil
// ```
//
// Here we search for members. This only works well if the list is
// not being inferred, of course.
?A in_list: [?A, ?L].
?A in_list: [?B, ?L] :- ?A in_list: ?L.
?A in_list: [?L, ?B] :- ?A in_list: ?L.
