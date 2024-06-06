#!/bin/bash

# Create a new README.md file if it doesn't exist
touch README.md

for i in {1..100000}
do
    # Append '/' to README.md
    echo '/' >> README.md

    # Add changes to git
    git add README.md

    # Commit changes with a message
    git commit -m "Add '/' character - iteration $i"
done

# Push the final result to the remote repository
git push origin main

echo "Completed 100,000 iterations and pushed to remote repository."