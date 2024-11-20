# rust_test
Test task

The solutions must be provided in Rust. Please mention all your steps and explain what led you
to choose your solution. You can briefly comment on other solutions and ideas which you had while solving this task.

Part 1: Self-Evaluation
On a scale from 1 to 10, rate your experience with Rust and how idiomatic your Rust code typically is.

ANSWER:

I would rate my Rust knowledge as a 5. I truly love this programming language and try to use it whenever I get the chance.
But unfortunately, I don't have the opportunity to practice Rust regularly, 
and the knowledge I acquire tends to fade quickly without consistent use.

I really enjoy working with types like Result and Option, which I find invaluable, especially compared to C++, 
where I miss such constructs. I transitioned from Scala relatively recently, 
and I appreciate how Rust offers a functional, safe, and expressive approach to programming.

I try to optimize my code with lifetime annotations when applicable and make it more generic by leveraging traits and parametric types. 
I avoid using unsafe unless it's absolutely necessary. 
I also aim to follow Rust's standard code style and use tools like cargo fmt to keep my code clean.

Whenever possible, I prefer using iterators, map, and similar functional constructs. 
However, I don't have much practical experience working with Rust in a team setting alongside more experienced developers.

Despite these limitations, I love Rust because it aligns perfectly with my vision of the ideal programming language. 
It provides expressive, functional, and safe code with excellent compile-time checks, making it a joy to use.

TODO: -----------------------------

Part 2: Data Structures and Algorithms
Create a data-set of words from the book https://www.gutenberg.org/files/98/98-0.txt. 
Implement a fixed sized open addressing hash table by using linear probing to resolve collisions. 
Assume that the keys are the words from the given data-set and the hash table’s values are integers. 
You need to implement the following functions with O(1)-complexity:

> The solutions must be provided in Rust. Please mention all your steps and explain what led you
to choose your solution. You can briefly comment on other solutions and ideas which you had while solving this task.

First, I tried to implement a basic, simple FixedHashTable with methods similar to the standard HashMap, 
using straightforward tools: a "Vec" for elements and a hash function to determine the index.

Next, I started adding some unit tests and immediately discovered a bug in the search functionality after deletions.
I guessed that adding a Deleted marker would help, and at the same time, 
I began jotting down TODOs and ideas for improvements that came to mind.

Afterward, I noticed that methods like get_last and get_first were also needed. 
After some thought, I realized a list of keys was necessary. 
I couldn't find a ready-made doubly linked list that met my requirements, 
so I quickly implemented a simple one myself. 
The elements had to be wrapped in Rc<RefCell> because references to nodes needed to be stored in multiple places. 
While this isn't the most optimal solution, it is the simplest.

Another option would have been to use unsafe or look for more efficient but equally safe approaches. 
However, time was limited, so I went with this compromise.

To avoid cloning string keys and incurring additional memory overhead, 
I decided to wrap the keys in the list and the FixedHashTable in Rc as well.

Using these components, I attempted to solve the primary task, though not all requirements were clearly defined.

1. What size should the container be?
I chose a small size 100. It seemed reasonable to consider real-world constraints, 
such as memory, since the number of unique words could be very large and not to big for a manual test. 
Additionally, using a size large enough to store all possible words might be overkill if we specifically aim for a fixed-size container.

2. Which words should be removed when the container is full?
It seemed logical to keep a slice of the freshest data, so when the container reached its limit, 
I decided to remove the oldest element.

After completing the primary task, I added more tests and addressed some TODOs, 
such as implementing rehashing when Deleted slots accumulate, making minor optimizations, using the RandomState hasher, and more.

What I would like to improve:
1. There are still TODOs in the code.
2. The FixedHashTable implementation is somewhat intertwined with DoublyLinkedList. It would be better to separate FixedHashTable and create an adapter that combines the features of both classes.
Write benchmark tests and profile the code.
3. Explore more efficient ways to implement the DoublyLinkedList.
4. Get a code review from a more experienced Rust developer and improve the implementation.

Part 3: Trading Specific Algorithms
Review the Binance European Options API documentation at https://binance-docs.github.io/apidocs/voptions/en/.

> The solutions must be provided in Rust. Please mention all your steps and explain what led you
to choose your solution. You can briefly comment on other solutions and ideas which you had while solving this task.

