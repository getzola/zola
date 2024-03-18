+++
title = "AWS S3 Bucket"
weight = 80
+++

Amazon Simple Storage Service (Amazon S3) is an object storage service offering static website hosting. We're going to look at the setup required to build and deploy your Zola website to S3 via GitHub Actions.

## AWS Setup

[The official AWS developer](https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/getting-started-cloudfront-overview.html) guide has detailed instruction on how to create your bucket and set it up correctly for static website hosting. In AWS you can not only host the website files, but also buy a domain name and speed up your website via their global CDN (CloudFront).

For GitHub Actions to modify the files in your bucket, you need to create an IAM user in your AWS account that has just enough permissions to perform what we need and no more.

First we need to create a new policy by logging on to AWS Console and going to **IAM** > **Policies** > **Create policy**. Switch from the visual editor to **JSON** and paste the following snippet. Remember to update your bucket name:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "AccessToWebsiteBuckets",
      "Effect": "Allow",
      "Action": [
        "s3:PutBucketWebsite",
        "s3:PutObject",
        "s3:PutObjectAcl",
        "s3:GetObject",
        "s3:ListBucket",
        "s3:DeleteObject"
      ],
      "Resource": [
        "arn:aws:s3:::Bucket-Name"
        "arn:aws:s3:::Bucket-Name/*"
      ]
    },
    {
      "Sid": "AccessToCloudfront",
      "Effect": "Allow",
      "Action": ["cloudfront:GetInvalidation", "cloudfront:CreateInvalidation"],
      "Resource": "*"
    }
  ]
}
```

The `AccessToCloudfront` portion is not required if you're not going to speed up your website with CloudFront.

Once the policy is created you need to create a new user under **IAM** > **Users**. Give it a name such as `github-actions-user`. On the **Set permissions** step select **Attach policies directly** and find the policy we created in the last step. 

From the list of users click on your newly created account and then open the **Security Credentials** tab. Under **Access keys** select > **Create access key** and choose **Command Line Interface (CLI)**. Click "I understand the above recommendation" and then **Create access key**. Note the **Access key ID** and **Secret access key**.

## Setup Secrets in GitHub

The access keys we just created need to be configured as secrets in your GitHub repo. To do so, navigate to **Setting** > expand **Secrets and variables** > click on **Actions**.

Under **Repository secrets** click **Add repository secret**. In the *Name* field enter `AWS_ACCESS_KEY_ID` and in the  *Secret* field enter the value from the previous step. Do the same for the secret access key, naming it `AWS_SECRET_ACCESS_KEY`. Finally create one secret for your bucket name `S3_BUCKET` and one `CLOUDFRONT_DISTRIBUTION_ID` if you have created a distribution for your website.

## GitHub Actions

Next we need to create the *Github Action* to build and deploy our files to S3. We need to create a workflow file in `.github/workflows` directory of our repository. This can be done by navigating to the *Actions* tab in GitHub or by commiting the file from your machine.

`.github/workflows/publish.yml`:

```yaml
name: Build and Publish to AWS
on:
  push:
    branches:
      - main
jobs:
  run:
    runs-on: ubuntu-latest
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@v3
      - uses: taiki-e/install-action@v2
        with:
          tool: zola@0.17.2
      - name: Build
        run: zola build
      - uses: reggionick/s3-deploy@v4
        env:
          AWS_ACCESS_KEY_ID: ${{ secrets.AWS_ACCESS_KEY_ID }}
          AWS_SECRET_ACCESS_KEY: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
        with:
          folder: public
          bucket: ${{ secrets.S3_BUCKET }}
          private: true
          bucket-region: us-east-1
          # Use the next two only if you have created a CloudFront distribution
          dist-id: ${{ secrets.CLOUDFRONT_DISTRIBUTION_ID }}
          invalidation: /*
```

Note, that you may need to change the branch name in the above snippet if you desire a different behavior.
