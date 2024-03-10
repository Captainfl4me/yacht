using UnityEngine;

public class PickableDice : InteractiveObject
{
    private Vector3 previousPosition;
    private Quaternion previousRotation;

    protected override void Awake()
    {
        base.Awake();
        base.interactiveType = InteractiveType.Pick;
    }

    public override bool CanInteract()
    {
        return true;
    }

    protected override void OnInteract()
    {
        previousPosition = transform.position;
        previousRotation = transform.rotation;
    }

    public void GoToPreviousPositionAndRotation()
    {
        transform.position = previousPosition;
        transform.rotation = previousRotation;
    }

    protected override void OnEndInteraction()
    {
    }
}
